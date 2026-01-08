#[cfg(test)]
mod test;

use {
    crate::{
        error::{
            InternalError,
            InternalResult,
            IntoInternal,
            OrElseUpgrade,
        },
        input::Input,
        macros::*,
        parser::Parser,
    },
    std::{
        fmt::Debug,
        io::{ Read, Write, },
        path::PathBuf,
    },
};

#[derive(Debug)]
pub(crate) enum FlowControl {
    Continue,
    Break,
}

pub(crate) type FlowResult = Result<FlowControl, InternalError>;
pub(crate) type StepResult<T> = Result<T, FlowResult>;

pub(crate) trait IntoStep<T> {
    fn into_step(self) -> StepResult<T>;
}

impl<T> IntoStep<T> for InternalResult<T> {
    fn into_step(self) -> StepResult<T> {
        self.map_err(Err)
    }
}

impl<T> OrElseUpgrade<T> for StepResult<T> {
    type Output = Self;

    fn or_else_upgrade<R, W, P>(self, parser: &P) -> Self
    where
        R: Read + Debug,
        W: Write + Debug,
        P: Parser<R, W>,
    {
        self.map_err(|e| e.map_err(|mut e| {
            e.upgrade(parser);
            e
        }))
    }

    fn or_else_upgrade_from_input<R>(self, input: &Input<R>) -> Self::Output
    where
        R: Read + Debug
    {
        self.map_err(|e| e.map_err(|mut e| {
            e.upgrade_from_input(input);
            e
        }))
    }
}

macro_rules! flow_ctrl {
    ($flow:expr) => {
        match $flow {
            FlowControl::Break => break,
            FlowControl::Continue => continue,
        }
    }
}

pub(crate) use flow_ctrl;

macro_rules! flow {
    ($to_result:expr) => {
        match $to_result {
            Ok(c) => c,
            Err(e) => match e {
                Ok(ctrl) => flow_ctrl!(ctrl),
                Err(e) => return Err(Err(e)),
            },
        }
    }
}

macro_rules! flow_internal {
    ($to_result:expr) => {
        match $to_result {
            Ok(c) => c,
            Err(e) => match e {
                Ok(ctrl) => flow_ctrl!(ctrl),
                Err(e) => return Err(e),
            },
        }
    }
}

pub(crate) use flow_internal;

pub(crate) trait Steps<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
    Self: Parser<R, W>,
{
    fn current_internal(&self) -> InternalResult<Option<char>> {
        match self.input()?.current() {
            Some(c) => Ok(Some(*c)),
            None => Ok(None),
        }
    }

    fn current(&self) -> StepResult<Option<char>> {
        self.current_internal().into_step()
    }

    fn current_or_continue(&mut self) -> StepResult<char> {
        match self.current()? {
            Some(c) => Ok(c),
            None => {
                self.output_mut().into_step()?.flush_buffer_to_content();
                Err(Ok(FlowControl::Continue))
            },
        }
    }

    fn current_or_break(&self) -> StepResult<char> {
        match self.current()? {
            Some(c) => Ok(c),
            None => {
                Err(Ok(FlowControl::Break))
            },
        }
    }

    fn unexpected_eof_internal(&self) -> InternalResult<()> {
        Err(InternalError::new("Unexpected EOF"))
    }

    fn push_step_internal(&mut self) -> InternalResult<()> {
        let c = match self.current_internal()? {
            Some(c) => c,
            None => return self.unexpected_eof_internal(),
        };

        self.output_mut()?.write_char(c);
        self.input_mut()?.step()?;

        Ok(())
    }

    fn push_step(&mut self) -> StepResult<()> {
        self.push_step_internal().into_step()
    }

    fn buffer_all_while<F>(&mut self, matches: F) -> StepResult<()>
    where
        F: Fn(char) -> bool,
    {
        loop {
            let c = flow!(self.current_or_break());

            if matches(c) {
                self.output_mut().into_step()?.write_char(c);
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }
            else {
                break;
            }
        }

        Ok(())
    }

    fn buffer_rest_of_tagname(&mut self) -> StepResult<String> {
        self.buffer_all_while(|c| matches!(c, first_variable_chars!()))?;

        String::from_utf8(self.output_mut().into_step()?.take_buffer())
            .into_internal("Invalid utf8 in tag name")
            .into_step()
    }

    fn unknown_tag<T>(&mut self) -> StepResult<T> {
        let tagname = self.buffer_rest_of_tagname()?;
        Err(Err(InternalError::new(format!("Unknown tag '{tagname}'"))))
    }

    fn unexpected_tag<T>(&mut self) -> StepResult<T> {
        let tagname = self.buffer_rest_of_tagname()?;
        Err(Err(InternalError::new(format!("Unexpected tag '{tagname}"))))
    }

    fn unknown_end_tag<T>(&mut self) -> StepResult<T> {
        let tagname = self.buffer_rest_of_tagname()?;
        Err(Err(InternalError::new(format!("Unknown end-tag '{tagname}"))))
    }

    fn unexpected_end_tag<T>(&mut self) -> StepResult<T> {
        let tagname = self.buffer_rest_of_tagname()?;
        Err(Err(InternalError::new(format!("Unexpected end-tag '{tagname}"))))
    }

    fn unexpected_eof_in_tag<T>(&mut self) -> StepResult<T> {
        let tagname = self.buffer_rest_of_tagname()?;
        Err(Err(InternalError::new(format!("Unexpected EOF in tag '{tagname}"))))
    }

    fn current_or_unexpected_eof_in_tag(&mut self) -> StepResult<char> {
        match self.current()? {
            Some(c) => Ok(c),
            None => self.unexpected_eof_in_tag(),
        }
    }

    fn tag_unexpected_eof<S, T>(&self, tagname: S) -> StepResult<T>
    where
        S: AsRef<str>,
    {
        Err(Err(InternalError::new(format!("Unexpected EOF in tag '{}'", tagname.as_ref()))))
    }

    fn tag_current_or_unexpected_eof<S>(&self, tagname: S) -> StepResult<char>
    where
        S: AsRef<str>,
    {
        match self.current()? {
            Some(c) => Ok(c),
            None => self.tag_unexpected_eof(tagname),
        }
    }

    fn tag_unexpected_eof_expected<S, S2, T>(&self, tagname: S, expected: S2) -> StepResult<T>
    where
        S: AsRef<str>,
        S2: AsRef<str>,
    {
        Err(Err(InternalError::new(format!(
            "Unexpected EOF in tag '{}', expected {}",
            tagname.as_ref(),
            expected.as_ref(),
        ))))
    }

    fn tag_unexpected_char<S, T>(&self, tagname: S) -> StepResult<T>
    where
        S: AsRef<str>,
    {
        Err(Err(InternalError::new(format!("Unexpected character in tag '{}'", tagname.as_ref()))))
    }

    fn tag_unexpected_char_expected<S, S2, T>(&self, tagname: S, expected: S2) -> StepResult<T>
    where
        S: AsRef<str>,
        S2: AsRef<str>,
    {
        Err(Err(InternalError::new(format!(
            "Unexpected character in tag '{}', expected {}",
            tagname.as_ref(),
            expected.as_ref(),
        ))))
    }

    fn end_tag_unexpected_eof<S, T>(&self, tagname: S) -> StepResult<T>
    where
        S: AsRef<str>,
    {
        Err(Err(InternalError::new(format!("Unexpected EOF in end-tag '{}'", tagname.as_ref()))))
    }

    fn end_tag_unexpected_char<S, T>(&self, tagname: S) -> StepResult<T>
    where
        S: AsRef<str>,
    {
        Err(Err(InternalError::new(format!("Unexpected character in end-tag '{}'", tagname.as_ref()))))
    }

    fn bypass_whitespace(&mut self) -> StepResult<()> {
        loop {
            let c = flow!(self.current_or_break());

            if c.is_whitespace() {
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }

            break;
        }

        Ok(())
    }

    fn buffer_whitespace(&mut self) -> StepResult<()> {
        loop {
            let c = flow!(self.current_or_break());

            if c.is_whitespace() {
                self.output_mut().into_step()?.write_char(c);
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }

            break;
        }

        Ok(())
    }

    fn bypass_whitespace_enforce_one(&mut self) -> StepResult<bool> {
        let c = match self.current()? {
            Some(c) => c,
            None => return Ok(false),
        };

        if c.is_whitespace() {
            self.bypass_whitespace()?;

            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    fn buffer_whitespace_enforce_one(&mut self) -> StepResult<bool> {
        let c = match self.current()? {
            Some(c) => c,
            None => return Ok(false),
        };

        if c.is_whitespace() {
            self.buffer_whitespace()?;

            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    fn buffer_all_until<F>(&mut self, matches: F) -> StepResult<()>
    where
        F: Fn(char) -> bool,
    {
        loop {
            let c = flow!(self.current_or_break());

            if matches(c) {
                break;
            }
            else {
                self.output_mut().into_step()?.write_char(c);
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }
        }

        Ok(())
    }

    fn buffer_all_until_sequence<S>(&mut self, tagname: S, seq: &[char]) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        let tagname = tagname.as_ref();

        loop {
            let c = flow!(self.current_or_break());
            if c == '\\' {
                self.output_mut().into_step()?.write_char(c);
                self.input_mut().into_step()?.step().into_step()?;

                let d = match self.current()? {
                    Some(d) => d,
                    None => return self.tag_unexpected_eof(tagname),
                };

                self.output_mut().into_step()?.write_char(d);
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }

            let mut flow = None;
            for ch in seq.iter() {
                let c = match self.current()? {
                    Some(c) => c,
                    None => return self.tag_unexpected_eof(tagname),
                };

                self.output_mut().into_step()?.write_char(c);
                self.input_mut().into_step()?.step().into_step()?;
                if c != *ch {
                    flow = Some(FlowControl::Continue);
                    break;
                }
            }

            match flow {
                Some(f) => flow_ctrl!(f),
                None => break,
            }
        }

        Ok(())
    }

    fn buffer_all_until_end_of_self_closing_tag<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.buffer_all_until_sequence(tagname, &['/', '%', '}'])
    }

    fn buffer_all_until_end_of_tag<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.buffer_all_until_sequence(tagname, &['%', '}'])
    }

    fn bypass_all_until<F>(&mut self, matches: F) -> StepResult<()>
    where
        F: Fn(char) -> bool,
    {
        loop {
            let c = flow!(self.current_or_break());

            if matches(c) {
                break;
            }
            else {
                self.input_mut().into_step()?.step().into_step()?;
                continue;
            }
        }

        Ok(())
    }

    fn tag_expect_char_internal<F, S>(&self, tagname: S, matches: F) -> StepResult<char>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        match self.current()? {
            Some(c) => if matches(c) {
                Ok(c)
            }
            else {
                self.tag_unexpected_char(tagname)
            },
            None => self.tag_unexpected_eof(tagname)
        }
    }

    fn tag_expect_char<F, S>(&mut self, tagname: S, matches: F) -> StepResult<()>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        self.tag_expect_char_internal(tagname, matches)?;
        self.input_mut().into_step()?.step().into_step()?;

        Ok(())
    }

    fn tag_expect_buffer_char<F, S>(&mut self, tagname: S, matches: F) -> StepResult<()>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        let c = self.tag_expect_char_internal(tagname, matches)?;
        self.output_mut().into_step()?.write_char(c);
        self.input_mut().into_step()?.step().into_step()?;

        Ok(())
    }

    fn end_tag_expect_char_internal<F, S>(&mut self, tagname: S, matches: F) -> StepResult<char>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        match self.current()? {
            Some(c) => if matches(c) {
                Ok(c)
            }
            else {
                self.end_tag_unexpected_char(tagname)
            },
            None => self.end_tag_unexpected_eof(tagname)
        }
    }

    fn end_tag_expect_char<F, S>(&mut self, tagname: S, matches: F) -> StepResult<()>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        self.end_tag_expect_char_internal(tagname, matches)?;
        self.input_mut().into_step()?.step().into_step()?;
        Ok(())
    }

    fn end_tag_expect_buffer_char<F, S>(&mut self, tagname: S, matches: F) -> StepResult<()>
    where
        S: AsRef<str>,
        F: FnOnce(char) -> bool,
    {
        let c = self.end_tag_expect_char_internal(tagname, matches)?;
        self.output_mut().into_step()?.write_char(c);
        self.input_mut().into_step()?.step().into_step()?;
        Ok(())
    }

    fn expect_end_of_self_close_tag<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.bypass_whitespace()?;

        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '/'))?;
        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '%'))?;
        self.tag_expect_char(tagname, |c| matches!(c, '}'))?;

        Ok(())
    }

    fn expect_end_of_tag<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.bypass_whitespace()?;

        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '%'))?;
        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '}'))?;

        Ok(())
    }

    fn expect_end_of_end_tag<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.bypass_whitespace()?;

        self.end_tag_expect_char(tagname.as_ref(), |c| matches!(c, '%'))?;
        self.end_tag_expect_char(tagname.as_ref(), |c| matches!(c, '}'))?;

        Ok(())
    }

    fn expect_end_of_tag_buffer<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.buffer_whitespace()?;

        self.tag_expect_buffer_char(tagname.as_ref(), |c| matches!(c, '%'))?;
        self.tag_expect_buffer_char(tagname.as_ref(), |c| matches!(c, '}'))?;

        Ok(())
    }

    fn expect_end_of_end_tag_buffer<S>(&mut self, tagname: S) -> StepResult<()>
    where
        S: AsRef<str>,
    {
        self.buffer_whitespace()?;

        self.end_tag_expect_buffer_char(tagname.as_ref(), |c| matches!(c, '%'))?;
        self.end_tag_expect_buffer_char(tagname.as_ref(), |c| matches!(c, '}'))?;

        Ok(())
    }

    fn parse_variable_name<S>(&mut self, tagname: S) -> StepResult<String>
    where
        S: AsRef<str>,
    {
        self.output_mut().into_step()?.clear_buffer();
        self.bypass_whitespace()?;

        self.tag_expect_buffer_char(tagname, |c| matches!(c, first_variable_chars!()))?;
        self.buffer_all_while(|c| matches!(c, variable_chars!()))?;

        let variable = String::from_utf8(self.output_mut().into_step()?.take_buffer())
            .into_internal("Invalid utf8 in variable name")
            .into_step()?;
        if variable.is_empty() {
            return Err(Err(InternalError::new("Variable name cannot be empty")));
        }

        Ok(variable)
    }

    fn parse_text_string<S>(&mut self, tagname: S) -> StepResult<String>
    where
        S: AsRef<str>,
    {
        self.output_mut().into_step()?.clear_buffer();
        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '"'))?;
        loop {
            self.buffer_all_until(|c| matches!(c, '"'|'\\'))?;
            let c = match self.current()? {
                Some(c) => c,
                None => return self.tag_unexpected_eof_expected(tagname, "\""),
            };

            match c {
                '"' => break,
                '\\' => {
                    self.input_mut().into_step()?.step().into_step()?;

                    let c = match self.current()? {
                        Some(c) => c,
                        None => return self.tag_unexpected_eof_expected(tagname, "\""),
                    };

                    match c {
                        '"' => {
                            self.output_mut().into_step()?.write_char(c);
                            self.input_mut().into_step()?.step().into_step()?;
                            continue;
                        },
                        _ => {
                            self.output_mut().into_step()?.write_char('\\');
                            self.output_mut().into_step()?.write_char(c);
                            self.input_mut().into_step()?.step().into_step()?;
                            continue;
                        },
                    }
                },
                _ => return Err(Err(InternalError::new("HOW!?"))),
            }
        }

        self.tag_expect_char(tagname, |c| matches!(c, '"'))?;

        String::from_utf8(self.output_mut().into_step()?.take_buffer())
            .into_internal("Invalid utf8 in text")
            .into_step()
    }

    fn parse_number<S>(&mut self, tagname: S) -> StepResult<String>
    where
        S: AsRef<str>,
    {
        self.output_mut().into_step()?.clear_buffer();
        self.buffer_all_while(|c| matches!(c, number_chars!()))?;
        String::from_utf8(self.output_mut().into_step()?.take_buffer())
            .into_internal(format!("Invalid UTF-8 in number literal of '{}' tag", tagname.as_ref()))
            .into_step()
    }

    fn parse_text<S>(&mut self, tagname: S) -> StepResult<String>
    where
        S: AsRef<str>,
    {
        self.parse_text_string(tagname)
    }

    fn parse_text_as_path<S>(&mut self, tagname: S) -> StepResult<PathBuf>
    where
        S: AsRef<str>,
    {
        let path = self.parse_text_string(tagname)?;
        Ok(self.input().into_step()?.join_path(path))
    }

    fn parse_variable<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Option<String>> {
        let alias = self.parse_variable_name(tagname)?;

        Ok(self.context().into_step()?.value(&alias).map(|v| v.to_owned()))
    }

    fn parse_variable_as_path<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Option<PathBuf>> {
        let alias = self.parse_variable_name(tagname)?;

        Ok(self.context().into_step()?.path(&alias))
    }

    fn parse_value<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Option<String>> {
        let c = self.tag_current_or_unexpected_eof(tagname.as_ref())?;

        match c {
            // string
            '"' => Ok(Some(self.parse_text(tagname)?)),
            // number
            number_chars!() => Ok(Some(self.parse_number(tagname)?)),
            // variable
            _ => self.parse_variable(tagname),
        }
     }

    fn parse_value_as_path<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Option<PathBuf>> {
        let c = self.tag_current_or_unexpected_eof(tagname.as_ref())?;

        match c {
            // string
            '"' => Ok(Some(self.parse_text_as_path(tagname)?)),
            // variable
            _ => self.parse_variable_as_path(tagname),
        }
     }

    fn parse_value_as_number<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<i64> {
        self.parse_value(tagname)?
            .into_internal("Cannot coerce an empty value into a number")
            .into_step()?
            .trim()
            .parse::<i64>()
            .into_internal("Failed to coerce value into a number")
            .into_step()
     }

    fn parse_function_args<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Vec<String>> {
        self.bypass_whitespace()?;
        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '('))?;
        self.output_mut().into_step()?.clear_buffer();

        let mut args = Vec::new();
        let mut first = true;
        while self.tag_current_or_unexpected_eof(tagname.as_ref())? != ')' {
            self.bypass_whitespace()?;

            if first {
                first = false;
            }
            else {
                self.tag_expect_char(tagname.as_ref(), |c| matches!(c, ','))?;
                self.bypass_whitespace()?;
            }

            args.push(self.parse_variable_name(tagname.as_ref())?);
            self.bypass_whitespace()?;
        }

        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, ')'))?;

        Ok(args)
    }

    fn parse_function_arg_values<S: AsRef<str>>(&mut self, tagname: S) -> StepResult<Vec<Option<String>>> {
        self.bypass_whitespace()?;
        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, '('))?;
        self.output_mut().into_step()?.clear_buffer();

        let mut args = Vec::new();
        let mut first = true;
        while self.tag_current_or_unexpected_eof(tagname.as_ref())? != ')' {
            self.bypass_whitespace()?;

            if first {
                first = false;
            }
            else {
                self.tag_expect_char(tagname.as_ref(), |c| matches!(c, ','))?;
                self.bypass_whitespace()?;
            }

            args.push(self.parse_value(tagname.as_ref())?);
            self.bypass_whitespace()?;
        }

        self.tag_expect_char(tagname.as_ref(), |c| matches!(c, ')'))?;

        Ok(args)
    }
}

impl<R, W, P> Steps<R, W> for P
where
    R: Read + Debug,
    W: Write + Debug,
    P: Parser<R, W>,
{}
