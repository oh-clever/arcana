#[cfg(test)]
mod test;

pub(crate) mod if_parser;
pub(crate) mod steps;

use {
    crate::{
        context::Context,
        error::{
            InternalError,
            InternalResult,
            IntoInternal,
            OrElseUpgrade,
        },
        input::{ Input, TryIntoInput },
        parser::{
            if_parser::IfParser,
            steps::*,
        },
        output::Output,
    },
    std::{
        fmt::Debug,
        fs::{ canonicalize, File, OpenOptions },
        io::{ Read, self, Write, },
        path::PathBuf,
    },
};

#[derive(Clone, Debug)]
pub(crate) enum ParseUntil {
    EndAdd,
    EndDiv,
    EndFn,
    EndForeach,
    EndFordir,
    EndForfile,
    EndForsplit,
    EndIf,
    EndMod,
    EndMul,
    EndNth,
    EndPow,
    EndSet,
    EndSub,
    Eof,
    // used exclusively by if tag
    ConditionEnd,
    Eot,
}

#[derive(Clone, Debug)]
pub(crate) enum EndPosition {
    Add,
    Else,
    Div,
    Fn,
    Foreach,
    Fordir,
    Forfile,
    Forsplit,
    Nth,
    If,
    Mod,
    Mul,
    Pow,
    Set,
    Sub,
    Eof,
}

pub(crate) trait Parser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    fn context(&self) -> InternalResult<&Context>;
    fn context_mut(&mut self) -> InternalResult<&mut Context>;
    fn give_context(&mut self, context: Option<Context>) -> Option<Context>;
    fn take_context(&mut self) -> InternalResult<Context>;

    fn input(&self) -> InternalResult<&Input<R>>;
    fn input_opt(&self) -> Option<&Input<R>>;
    fn input_mut(&mut self) -> InternalResult<&mut Input<R>>;
    fn give_input(&mut self, input: Option<Input<R>>) -> Option<Input<R>>;
    fn take_input(&mut self) -> InternalResult<Input<R>>;

    fn output_mut(&mut self) -> InternalResult<&mut Output<W>>;
    fn give_output(&mut self, output: Option<Output<W>>) -> Option<Output<W>>;
    fn take_output(&mut self) -> InternalResult<Output<W>>;
}

#[derive(Debug)]
pub struct TemplateParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    extend: Option<PathBuf>,
    context: Option<Context>,
    input: Option<Input<R>>,
    parse_until: ParseUntil,
    bypass: bool,
    output: Option<Output<W>>,
    end_position: Option<EndPosition>,
}

impl<R, W> Parser<R, W> for TemplateParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    fn context(&self) -> InternalResult<&Context> {
        self.context.as_ref().into_internal("Context is None")
    }

    fn context_mut(&mut self) -> InternalResult<&mut Context> {
        self.context.as_mut().into_internal("Context is None")
    }

    fn give_context(&mut self, mut context: Option<Context>) -> Option<Context> {
        std::mem::swap(&mut context, &mut self.context);
        context
    }

    fn take_context(&mut self) -> InternalResult<Context> {
        self.give_context(None).into_internal("Context was None and could not be taken")
    }

    fn input(&self) -> InternalResult<&Input<R>> {
        self.input.as_ref().into_internal("Input is None")
    }

    fn input_opt(&self) -> Option<&Input<R>> {
        self.input.as_ref()
    }

    fn input_mut(&mut self) -> InternalResult<&mut Input<R>> {
        self.input.as_mut().into_internal("Input is None")
    }

    fn give_input(&mut self, mut input: Option<Input<R>>) -> Option<Input<R>> {
        std::mem::swap(&mut self.input, &mut input);
        input
    }

    fn take_input(&mut self) -> InternalResult<Input<R>> {
        self.give_input(None).into_internal("Input was None and could not be taken")
    }

    fn output_mut(&mut self) -> InternalResult<&mut Output<W>> {
        self.output.as_mut().into_internal("Output is None")
    }

    fn give_output(&mut self, mut output: Option<Output<W>>) -> Option<Output<W>> {
        std::mem::swap(&mut self.output, &mut output);
        output
    }

    fn take_output(&mut self) -> InternalResult<Output<W>> {
        self.give_output(None).into_internal("Output was None and could not be taken")
    }
}

impl<R, W> TemplateParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    pub(crate) fn bypass(&self) -> bool {
        self.bypass
    }

    pub(crate) fn parse_until(&self) -> &ParseUntil {
        &self.parse_until
    }

    pub(crate) fn set_end_position(&mut self, end_position: EndPosition) {
        self.end_position = Some(end_position);
    }

    pub(crate) fn set_extend(&mut self, path: PathBuf) {
        self.extend = Some(path);
    }

    fn new_internal<I, O>(
        context: Context, into_input: I, into_output: O,
        parse_until: ParseUntil, bypass: bool,
    ) -> InternalResult<Self>
    where
        I: TryIntoInput<R>,
        O: Into<Output<W>>,
    {
        let input = into_input.try_into_input()?;

        Ok(Self {
            extend: None,
            context: Some(context),
            input: Some(input),
            parse_until,
            bypass,
            output: Some(into_output.into()),
            end_position: None,
        })
    }

    pub fn new<I, O>(
        context: Context, into_input: I, into_output: O,
    ) -> InternalResult<Self>
    where
        I: TryIntoInput<R>,
        O: Into<Output<W>>,
    {
        Self::new_internal(
            context, into_input, into_output,
            ParseUntil::Eof, false,
        )
    }

    pub(crate) fn spawn_sealed(&mut self, path: PathBuf) -> InternalResult<TemplateParser<File, W>> {
        TemplateParser::new_internal(
            self.context()?.clone(),
            path,
            self.take_output()?,
            ParseUntil::Eof,
            self.bypass,
        )
    }

    pub(crate) fn consume_sealed(&mut self, mut sealed: TemplateParser<File, W>) -> InternalResult<()> {
        self.give_output(Some(sealed.take_output()?));

        Ok(())
    }

    pub(crate) fn parse_sealed(&mut self, path: PathBuf) -> InternalResult<()> {
        let mut sealed = self.spawn_sealed(path)?;
        sealed.parse_internal()?;
        self.consume_sealed(sealed)?;

        Ok(())
    }

    pub(crate) fn spawn_unsealed(&mut self, file: PathBuf) -> InternalResult<TemplateParser<File, W>> {
        self.output_mut()?.flush_buffer_to_content();

        TemplateParser::new_internal(
            self.take_context()?,
            file,
            self.take_output()?,
            ParseUntil::Eof,
            self.bypass,
        )
    }

    pub(crate) fn consume_unsealed(&mut self, mut unsealed: TemplateParser<File, W>) -> InternalResult<()> {
        self.give_context(Some(unsealed.take_context()?));
        self.give_output(Some(unsealed.take_output()?));

        Ok(())
    }

    pub(crate) fn parse_unsealed(&mut self, file: PathBuf) -> InternalResult<()> {
        let mut unsealed = self.spawn_unsealed(file)?;
        unsealed.parse_internal()?;
        self.consume_unsealed(unsealed)?;

        Ok(())
    }

    pub(crate) fn spawn_extend(&mut self, extend: PathBuf) -> InternalResult<TemplateParser<File, W>> {
        self.output_mut()?.flush_buffer_to_content();
        let content = String::from_utf8(self.output_mut()?.take_content())
            .into_internal("Invalid utf8 in content")?;

        let input_path = self.input()?.path().to_owned();
        self.context_mut()?.add_variable("CONTENT", input_path, content);

        TemplateParser::new_internal(
            self.take_context()?,
            extend,
            self.take_output()?,
            ParseUntil::Eof,
            self.bypass,
        )
    }

    pub(crate) fn consume_extend(&mut self, mut extend: TemplateParser<File, W>) -> InternalResult<()> {
        self.give_context(Some(extend.take_context()?));
        self.context_mut()?.remove_variable("CONTENT");
        self.give_output(Some(extend.take_output()?));

        Ok(())
    }

    pub(crate) fn parse_extend(&mut self, extend: PathBuf) -> InternalResult<()> {
        let mut extend = self.spawn_extend(extend)?;
        extend.parse_internal()?;
        self.consume_extend(extend)?;

        Ok(())
    }

    pub(crate) fn spawn_bypassed<'unsealed>(
        &mut self, output: &'unsealed mut Vec<u8>, parse_until: ParseUntil,
    ) -> InternalResult<TemplateParser<R, &'unsealed mut Vec<u8>>> {
        TemplateParser::new_internal(
            self.take_context()?,
            self.take_input()?,
            Output::from(output),
            parse_until,
            true,
        )
    }

    pub(crate) fn consume_bypassed(&mut self, mut bypassed: TemplateParser<R, &mut Vec<u8>>) -> InternalResult<EndPosition> {
        self.give_context(Some(bypassed.take_context()?));
        self.give_input(Some(bypassed.take_input()?));

        bypassed.end_position.take()
            .into_internal("End position was None")
    }

    pub(crate) fn parse_bypassed(&mut self, parse_until: ParseUntil) -> InternalResult<(Vec<u8>, EndPosition)> {
        let mut bytes = Vec::new();
        let mut bypassed = self.spawn_bypassed(&mut bytes, parse_until)?;
        bypassed.parse()?;
        let end_pos = self.consume_bypassed(bypassed)?;

        Ok((bytes, end_pos))
    }

    pub(crate) fn spawn_limited<'limited>(
        &mut self, input: Input<&'limited [u8]>, parse_until: ParseUntil
    ) -> InternalResult<TemplateParser<&'limited [u8], W>> {
        TemplateParser::new_internal(
            self.take_context()?,
            input,
            self.take_output()?,
            parse_until,
            self.bypass,
        )
    }

    pub(crate) fn consume_limited(&mut self, mut limited: TemplateParser<&[u8], W>) -> InternalResult<()> {
        self.give_context(Some(limited.take_context()?));
        self.give_output(Some(limited.take_output()?));

        Ok(())
    }

    pub(crate) fn parse_limited<'limited, I>(
        &mut self, into_input: I, parse_until: ParseUntil
    ) -> InternalResult<()>
    where
        I: TryIntoInput<&'limited [u8]>,
    {
        // make sure input has the same path as it is within the same file
        let mut input = into_input.try_into_input()?;
        input.set_path(self.input()?.path());

        let mut limited = self.spawn_limited(
            input,
            parse_until
        )?;
        limited.parse_internal()?;
        self.consume_limited(limited)?;

        Ok(())
    }

    pub(crate) fn spawn_limited_sealed<'limited>(
        &mut self, context: Context, input: Input<&'limited [u8]>, parse_until: ParseUntil
    ) -> InternalResult<TemplateParser<&'limited [u8], W>> {
        TemplateParser::new_internal(
            context,
            input,
            self.take_output()?,
            parse_until,
            self.bypass,
        )
    }

    pub(crate) fn consume_limited_sealed(&mut self, mut limited: TemplateParser<&[u8], W>) -> InternalResult<()> {
        self.give_output(Some(limited.take_output()?));

        Ok(())
    }

    pub(crate) fn parse_limited_sealed<'limited, I>(
        &mut self, context: Context, into_input: I, parse_until: ParseUntil
    ) -> InternalResult<()>
    where
        I: TryIntoInput<&'limited [u8]>,
    {
        // make sure input has the same path as it is within the same file
        let mut input = into_input.try_into_input()?;
        input.set_path(self.input()?.path());

        let mut limited = self.spawn_limited_sealed(context, input, parse_until)?;
        limited.parse_internal()?;
        self.consume_limited_sealed(limited)?;

        Ok(())
    }

    pub(crate) fn spawn_child<O2: Write + Debug, W2: Into<Output<O2>>>(
        &mut self, output: W2, parse_until: ParseUntil
    ) -> InternalResult<TemplateParser<R, O2>> {
        TemplateParser::new_internal(
            self.take_context()?,
            self.take_input()?,
            output,
            parse_until,
            self.bypass,
        )
    }

    pub(crate) fn consume_child<O2: Write + Debug>(&mut self, mut child: TemplateParser<R, O2>) -> InternalResult<()> {
        self.give_context(Some(child.take_context()?));
        self.give_input(Some(child.take_input()?));

        Ok(())
    }

    pub(crate) fn parse_child(&mut self, parse_until: ParseUntil) -> InternalResult<String> {
        let mut output_bytes = Vec::new();
        let mut child = self.spawn_child(&mut output_bytes, parse_until)?;
        child.parse_internal()?;
        child.write()?;
        self.consume_child(child)?;

        let output_string = String::from_utf8(output_bytes)
            .into_internal("Invalid utf-8 found in output")?;

        Ok(output_string)
    }

    fn parse_add(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("add")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndAdd)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Add => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'add' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("add")?;

            self.expect_end_of_tag("add")?;

            let content = self.parse_child(ParseUntil::EndAdd).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            self.output_mut().into_step()?.write_str(&(value + content).to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_ad(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_add()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_assert(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("assert")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let condition = IfParser::parse_result("assert", self)?;

            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;
            self.expect_end_of_self_close_tag("assert")?;

            if !condition.as_evaluation() {
                return Err(Err(InternalError::new(
                    "ASSERTION FAILED"
                )));
            }

            Ok(())
        }
    }

    fn parse_asser(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_assert()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_asse(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_asser()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ass(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_asse()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_as(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            's' => {
                self.push_step()?;
                self.parse_ass()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_a(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_ad()
            },
            's' => {
                self.push_step()?;
                self.parse_as()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_basename(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("basename")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let path = self.parse_value("basename")?
                .into_internal("Cannot canonicalize a path from an empty value")
                .into_step()?;

            self.bypass_whitespace()?;

            let mut reldir = if self.tag_current_or_unexpected_eof("basename")? == 'i' {
                self.push_step()?;

                self.tag_expect_char("basename", |c| matches!(c, 'n'))?;
                self.bypass_whitespace()?;

                self.output_mut().into_step()?.clear_buffer();

                self.parse_value_as_path("basename")?
                    .into_internal("Cannot create an absolute path from a None path")
                    .into_step()?
            }
            else {
                let mut input_path = self.input().into_step()?.path().to_owned();
                input_path.pop();
                input_path
            };

            reldir.push(path);

            let basename = canonicalize(&reldir)
                .into_internal(format!("Failed to canonicalize relative path {reldir:?}"))
                .into_step()?
                .file_name()
                .map(|osstr| osstr.to_str().unwrap_or("").to_owned())
                .unwrap_or_else(String::new);

            self.bypass_whitespace()?;

            self.output_mut().into_step()?.clear_buffer();
            self.output_mut().into_step()?.write_str(&basename);
            self.output_mut().into_step()?.flush_buffer_to_content();

            self.expect_end_of_self_close_tag("basename")?;

            Ok(())
        }
    }

    fn parse_basenam(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_basename()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_basena(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'm' => {
                self.push_step()?;
                self.parse_basenam()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_basen(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_basena()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_base(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_basen()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_bas(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_base()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ba(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            's' => {
                self.push_step()?;
                self.parse_bas()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_b(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_ba()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_call(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("call")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            let path = self.parse_value_as_path("call")?
                .into_internal("Path was None and this message should be better")
                .into_step()?;
            self.expect_end_of_self_close_tag("call")?;

            // make sure we write all buffered content before spawning the sealed
            // parser
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.parse_unsealed(path).into_step()?;

            Ok(())
        }
    }

    fn parse_cal(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_call()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ca(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_cal()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_compile(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("compile")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            let path = self.parse_value_as_path("compile")?
                .into_internal("Cannot canonicalize an empty value")
                .into_step()?;
            self.expect_end_of_self_close_tag("compile")?;

            // make sure we write all buffered content before spawning the sealed
            // parser
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.parse_sealed(path).into_step()?;

            Ok(())
        }
    }

    fn parse_compil(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_compile()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_compi(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_compil()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_comp(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_compi()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_com(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'p' => {
                self.push_step()?;
                self.parse_comp()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_count(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("count")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();
            let alias = self.parse_variable_name("count")?;
            let count = self.context().into_step()?
                .values(&alias)
                .unwrap_or(vec![])
                .len();
            self.expect_end_of_self_close_tag("count")?;

            self.output_mut().into_step()?.write_str(&count.to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_coun(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_count()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_cou(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_coun()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_co(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'm' => {
                self.push_step()?;
                self.parse_com()
            },
            'u' => {
                self.push_step()?;
                self.parse_cou()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_c(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_ca()
            },
            'o' => {
                self.push_step()?;
                self.parse_co()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_dirname(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("dirname")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let path = self.parse_value("dirname")?
                .into_internal("Cannot canonicalize a path from an empty value")
                .into_step()?;

            self.bypass_whitespace()?;

            let mut reldir = if self.tag_current_or_unexpected_eof("dirname")? == 'i' {
                self.push_step()?;

                self.tag_expect_char("dirname", |c| matches!(c, 'n'))?;
                self.bypass_whitespace()?;

                self.output_mut().into_step()?.clear_buffer();

                self.parse_value_as_path("dirname")?
                    .into_internal("Cannot create an absolute path from a None path")
                    .into_step()?
            }
            else {
                let mut input_path = self.input().into_step()?.path().to_owned();
                input_path.pop();
                input_path
            };

            reldir.push(path);

            let dirname = canonicalize(&reldir)
                .into_internal(format!("Failed to canonicalize relative path {reldir:?}"))
                .into_step()?
                .parent()
                .map_or_else(
                    String::new,
                    |path| path.to_str()
                        .map(|s| s.to_owned())
                        .unwrap_or_else(String::new)
                );

            self.bypass_whitespace()?;

            self.output_mut().into_step()?.clear_buffer();
            self.output_mut().into_step()?.write_str(&dirname);
            self.output_mut().into_step()?.flush_buffer_to_content();

            self.expect_end_of_self_close_tag("dirname")?;

            Ok(())
        }
    }

    fn parse_dirnam(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_dirname()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_dirna(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'm' => {
                self.push_step()?;
                self.parse_dirnam()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_dirn(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_dirna()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_dir(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_dirn()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_div(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("div")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndDiv)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Div => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'div' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("div")?;

            self.expect_end_of_tag("div")?;

            let content = self.parse_child(ParseUntil::EndDiv).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            self.output_mut().into_step()?
                .write_str(&value.checked_div(content).unwrap_or(0).to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_di(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_dir()
            },
            'v' => {
                self.push_step()?;
                self.parse_div()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_d(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_di()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_else(&mut self) -> StepResult<()> {
        if self.bypass() {
            match self.parse_until() {
                ParseUntil::EndFordir|
                ParseUntil::EndForeach|
                ParseUntil::EndForfile|
                ParseUntil::EndForsplit|
                ParseUntil::EndIf => {},
                _ => {
                    return self.unexpected_tag();
                },
            }

            self.expect_end_of_tag_buffer("else")?;
            self.set_end_position(EndPosition::Else);
            self.output_mut().into_step()?.flush_buffer_to_content();

            Err(Ok(FlowControl::Break))
        }
        else {
            match self.parse_until() {
                ParseUntil::EndFordir|
                ParseUntil::EndForeach|
                ParseUntil::EndForfile|
                ParseUntil::EndForsplit|
                ParseUntil::EndIf => {},
                _ => {
                    return self.unexpected_tag();
                },
            }

            self.output_mut().into_step()?.clear_buffer();
            self.expect_end_of_tag("else")?;
            self.set_end_position(EndPosition::Else);

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_els(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_else()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_el(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            's' => {
                self.push_step()?;
                self.parse_els()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_extend_tag(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("extend")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            let path = self.parse_value_as_path("extend")?
                .into_internal("The extended path was None and this message sucks")
                .into_step()?;
            self.set_extend(path);

            self.expect_end_of_self_close_tag("extend")?;

            Ok(())
        }
    }

    fn parse_exten(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_extend_tag()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_exte(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_exten()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ext(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_exte()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ex(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_ext()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_e(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_el()
            },
            'x' => {
                self.push_step()?;
                self.parse_ex()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_fordir(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("fordir")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndFordir)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndFordir)
                        .into_step()?;
                    self.output_mut().into_step()?.write_bytes_to_buffer(else_content);
                },
                EndPosition::Fordir => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'fordir' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let variable = self.parse_variable_name("fordir")?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'fordir' tag")));
            }

            self.tag_expect_char("fordir", |c| matches!(c, 'i'))?;
            self.tag_expect_char("fordir", |c| matches!(c, 'n'))?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'fordir' tag")));
            }

            let path = self.parse_value_as_path("fordir")?
                .into_internal("Cannot iterate over directories within a None path")
                .into_step()?;

            self.bypass_whitespace()?;

            let mut from_idx = None;
            if self.tag_current_or_unexpected_eof("fordir")? == 'f' {
                self.push_step()?;
                self.tag_expect_buffer_char("fordir", |c| matches!(c, 'r'))?;
                self.tag_expect_buffer_char("fordir", |c| matches!(c, 'o'))?;
                self.tag_expect_buffer_char("fordir", |c| matches!(c, 'm'))?;

                self.bypass_whitespace()?;
                from_idx = Some(self.parse_value_as_number("fordir")?);
                self.bypass_whitespace()?;
            }

            let mut to_idx = None;
            if self.tag_current_or_unexpected_eof("fordir")? == 't' {
                self.push_step()?;
                self.tag_expect_buffer_char("fordir", |c| matches!(c, 'o'))?;

                self.bypass_whitespace()?;
                to_idx = Some(self.parse_value_as_number("fordir")?);
                self.bypass_whitespace()?;
            }

            let mut loop_variable = None;
            if self.tag_current_or_unexpected_eof("fordir")? == 'a' {
                self.push_step()?;

                match self.tag_current_or_unexpected_eof("fordir")? {
                    's' => {
                        self.push_step()?;

                        self.bypass_whitespace()?;

                        loop_variable = Some(self.parse_variable_name("fordir")?);

                        self.bypass_whitespace()?;
                    },
                    _ => return self.tag_unexpected_char("fordir"),
                }
            }

            let reversed = if self.tag_current_or_unexpected_eof("fordir")? == 'r' {
                self.push_step()?;

                self.tag_expect_buffer_char("fordir", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'v'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'r'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'s'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("fordir", |c| c.eq(&'d'))?;

                self.output_mut().into_step()?.clear_buffer();
                self.bypass_whitespace()?;

                true
            }
            else {
                false
            };

            self.expect_end_of_tag("fordir")?;

            let mut dirpaths = path.read_dir()
                .into_internal("Failed to read directory")
                .into_step()?
                .map(|direntry_res| direntry_res.map(|de| de.path()))
                .collect::<io::Result<Vec<PathBuf>>>()
                .into_internal("Failed to read paths of directory entries")
                .into_step()?
                .into_iter()
                .filter(|direntry| direntry.is_dir())
                .collect::<Vec<PathBuf>>();

            dirpaths.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));

            if from_idx.is_some() || to_idx.is_some() {
                let min = from_idx.unwrap_or(0_i64);
                let max = to_idx.unwrap_or(dirpaths.len() as i64);

                dirpaths = dirpaths.into_iter()
                    .enumerate()
                    .filter(|(i, _)| (i.to_owned() as i64) >= min)
                    .filter(|(i, _)| (i.to_owned() as i64) < max)
                    .map(|(_, v)| v)
                    .collect::<Vec<PathBuf>>();
            }

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndFordir)
                .into_step()?;
            let else_content = match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndFordir)
                        .into_step()?;
                    Some(else_content)
                },
                EndPosition::Fordir => None,
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'fordir' tag, '{pos:?}'"
                )))),
            };

            if !dirpaths.is_empty() {
                let last = dirpaths.len();

                if reversed {
                    dirpaths.reverse();
                }

                let path = self.input().into_step()?.path().to_owned();
                for (index, dirpath) in dirpaths.into_iter().enumerate() {
                    let dirstr = dirpath.to_str().unwrap_or("").to_owned();
                    self.context_mut().into_step()?.add_variable(&variable, &path, dirstr);

                    if let Some(loop_variable) = loop_variable.clone() {
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.index"), &path, index.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.size"), &path, last.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.isfirst"), &path, if index == 0 { "1" } else { "0" });
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.islast"), &path, if index + 1 == last { "1" } else { "0" });
                    }

                    self.parse_limited(
                        content.as_slice(),
                        ParseUntil::EndFordir
                    ).into_step()?;

                    self.context_mut().into_step()?.pop_variable(&variable);
                }
            }
            else if let Some(content) = else_content {
                self.parse_limited(content.as_slice(), ParseUntil::EndFordir)
                    .into_step()?;
            }

            Ok(())
        }
    }

    fn parse_fordi(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_fordir()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_ford(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_fordi()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_foreach(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("foreach")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForeach)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForeach)
                        .into_step()?;
                    self.output_mut().into_step()?.write_bytes_to_buffer(else_content);
                },
                EndPosition::Foreach => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'foreach' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let variable = self.parse_variable_name("foreach")?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'foreach' tag")));
            }

            self.tag_expect_char("foreach", |c| matches!(c, 'i'))?;
            self.tag_expect_char("foreach", |c| matches!(c, 'n'))?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'foreach' tag")));
            }

            let alias = self.parse_variable_name("foreach")?;

            self.bypass_whitespace()?;

            let mut from_idx = None;
            if self.tag_current_or_unexpected_eof("foreach")? == 'f' {
                self.push_step()?;
                self.tag_expect_buffer_char("foreach", |c| matches!(c, 'r'))?;
                self.tag_expect_buffer_char("foreach", |c| matches!(c, 'o'))?;
                self.tag_expect_buffer_char("foreach", |c| matches!(c, 'm'))?;

                self.bypass_whitespace()?;
                from_idx = Some(self.parse_value_as_number("foreach")?);
                self.bypass_whitespace()?;
            }

            let mut to_idx = None;
            if self.tag_current_or_unexpected_eof("foreach")? == 't' {
                self.push_step()?;
                self.tag_expect_buffer_char("foreach", |c| matches!(c, 'o'))?;

                self.bypass_whitespace()?;
                to_idx = Some(self.parse_value_as_number("foreach")?);
                self.bypass_whitespace()?;
            }

            let mut loop_variable = None;
            if self.tag_current_or_unexpected_eof("foreach")? == 'a' {
                self.push_step()?;

                self.tag_expect_buffer_char("foreach", |c| matches!(c, 's'))?;

                self.bypass_whitespace()?;
                loop_variable = Some(self.parse_variable_name("foreach")?);
                self.bypass_whitespace()?;
            }

            let reversed = if self.tag_current_or_unexpected_eof("foreach")? == 'r' {
                self.push_step()?;

                self.tag_expect_buffer_char("foreach", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'v'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'r'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'s'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("foreach", |c| c.eq(&'d'))?;

                self.output_mut().into_step()?.clear_buffer();
                self.bypass_whitespace()?;

                true
            }
            else {
                false
            };

            self.expect_end_of_tag("foreach")?;

            let mut values = self.context().into_step()?
                .values(&alias)
                .unwrap_or(vec![])
                .into_iter()
                .map(|v| v.to_owned())
                .collect::<Vec<String>>();

            if from_idx.is_some() || to_idx.is_some() {
                let min = from_idx.unwrap_or(0_i64);
                let max = to_idx.unwrap_or(values.len() as i64);

                values = values.into_iter()
                    .enumerate()
                    .filter(|(i, _)| (i.to_owned() as i64) >= min)
                    .filter(|(i, _)| (i.to_owned() as i64) < max)
                    .map(|(_, v)| v)
                    .collect::<Vec<String>>();
            }

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForeach)
                .into_step()?;
            let else_content = match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForeach)
                        .into_step()?;
                    Some(else_content)
                },
                EndPosition::Foreach => None,
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'foreach' tag, '{pos:?}'"
                )))),
            };

            if !values.is_empty() {
                let last = values.len();

                if reversed {
                    values.reverse();
                }

                let path = self.input().into_step()?.path().to_owned();
                for (index, value) in values.into_iter().enumerate() {
                    self.context_mut().into_step()?.add_variable(&variable, &path, value.clone());

                    if let Some(loop_variable) = loop_variable.clone() {
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.index"), &path, index.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.size"), &path, last.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.isfirst"), &path, if index == 0 { "1" } else { "0" });
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.islast"), &path, if index + 1 == last { "1" } else { "0" });
                    }

                    self.parse_limited(content.as_slice(), ParseUntil::EndForeach).into_step()?;

                    self.context_mut().into_step()?.pop_variable(&variable);
                }
            }
            else if let Some(content) = else_content {
                self.parse_limited(content.as_slice(), ParseUntil::EndForeach)
                    .into_step()?;
            }

            Ok(())
        }
    }

    fn parse_foreac(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_foreach()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forea(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'c' => {
                self.push_step()?;
                self.parse_foreac()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_fore(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_forea()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forfile(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("forfile")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForfile)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForfile)
                        .into_step()?;
                    self.output_mut().into_step()?.write_bytes_to_buffer(else_content);
                },
                EndPosition::Forfile => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'forfile' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let variable = self.parse_variable_name("forfile")?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'forfile' tag")));
            }

            self.tag_expect_char("forfile", |c| matches!(c, 'i'))?;
            self.tag_expect_char("forfile", |c| matches!(c, 'n'))?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'forfile' tag")));
            }

            let path = self.parse_value_as_path("forfile")?
                .into_internal("Cannot iterate over files within a None path")
                .into_step()?;

            self.bypass_whitespace()?;

            let mut from_idx = None;
            if self.tag_current_or_unexpected_eof("forfile")? == 'f' {
                self.push_step()?;
                self.tag_expect_buffer_char("forfile", |c| matches!(c, 'r'))?;
                self.tag_expect_buffer_char("forfile", |c| matches!(c, 'o'))?;
                self.tag_expect_buffer_char("forfile", |c| matches!(c, 'm'))?;

                self.bypass_whitespace()?;
                from_idx = Some(self.parse_value_as_number("forfile")?);
                self.bypass_whitespace()?;
            }

            let mut to_idx = None;
            if self.tag_current_or_unexpected_eof("forfile")? == 't' {
                self.push_step()?;
                self.tag_expect_buffer_char("forfile", |c| matches!(c, 'o'))?;

                self.bypass_whitespace()?;
                to_idx = Some(self.parse_value_as_number("forfile")?);
                self.bypass_whitespace()?;
            }

            let mut loop_variable = None;
            if self.tag_current_or_unexpected_eof("forfile")? == 'a' {
                self.push_step()?;

                match self.tag_current_or_unexpected_eof("forfile")? {
                    's' => {
                        self.push_step()?;

                        self.bypass_whitespace()?;

                        loop_variable = Some(self.parse_variable_name("forfile")?);

                        self.bypass_whitespace()?;
                    },
                    _ => return self.tag_unexpected_char("forfile"),
                }
            }

            let reversed = if self.tag_current_or_unexpected_eof("forfile")? == 'r' {
                self.push_step()?;

                self.tag_expect_buffer_char("forfile", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'v'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'r'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'s'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forfile", |c| c.eq(&'d'))?;

                self.output_mut().into_step()?.clear_buffer();
                self.bypass_whitespace()?;

                true
            }
            else {
                false
            };

            self.expect_end_of_tag("forfile")?;

            let mut filepaths = path.read_dir()
                .into_internal("Failed to read directory")
                .into_step()?
                .map(|direntry_res| direntry_res.map(|de| de.path()))
                .collect::<io::Result<Vec<PathBuf>>>()
                .into_internal("Failed to read paths of directory entries")
                .into_step()?
                .into_iter()
                .filter(|direntry| direntry.is_file())
                .collect::<Vec<PathBuf>>();

            if from_idx.is_some() || to_idx.is_some() {
                let min = from_idx.unwrap_or(0_i64);
                let max = to_idx.unwrap_or(filepaths.len() as i64);

                filepaths = filepaths.into_iter()
                    .enumerate()
                    .filter(|(i, _)| (i.to_owned() as i64) >= min)
                    .filter(|(i, _)| (i.to_owned() as i64) < max)
                    .map(|(_, v)| v)
                    .collect::<Vec<PathBuf>>();
            }

            filepaths.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForfile)
                .into_step()?;
            let else_content = match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForfile)
                        .into_step()?;
                    Some(else_content)
                },
                EndPosition::Forfile => None,
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'forfile' tag, '{pos:?}'"
                )))),
            };

            if !filepaths.is_empty() {
                let last = filepaths.len();

                if reversed {
                    filepaths.reverse();
                }

                let path = self.input().into_step()?.path().to_owned();
                for (index, filepath) in filepaths.into_iter().enumerate() {
                    let filestr = filepath.to_str().unwrap_or("").to_owned();
                    self.context_mut().into_step()?.add_variable(&variable, &path, filestr);

                    if let Some(loop_variable) = loop_variable.clone() {
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.index"), &path, index.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.size"), &path, last.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.isfirst"), &path, if index == 0 { "1" } else { "0" });
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.islast"), &path, if index + 1 == last { "1" } else { "0" });
                    }

                    self.parse_limited(
                        content.as_slice(),
                        ParseUntil::EndForfile
                    ).into_step()?;

                    self.context_mut().into_step()?.pop_variable(&variable);
                }
            }
            else if let Some(content) = else_content {
                self.parse_limited(content.as_slice(), ParseUntil::EndForfile)
                    .into_step()?;
            }

            Ok(())
        }
    }

    fn parse_forfil(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_forfile()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forfi(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_forfil()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forf(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_forfi()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forsplit(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("forsplit")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForsplit)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForsplit)
                        .into_step()?;
                    self.output_mut().into_step()?.write_bytes_to_buffer(else_content);
                },
                EndPosition::Forsplit => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'forsplit' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let variable = self.parse_variable_name("forsplit")?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'forsplit' tag")));
            }

            self.tag_expect_char("forsplit", |c| matches!(c, 'i'))?;
            self.tag_expect_char("forsplit", |c| matches!(c, 'n'))?;

            if !self.bypass_whitespace_enforce_one()? {
                return Err(Err(InternalError::new("Unexpected character in 'forsplit' tag")));
            }

            let str_value = self.parse_value("forsplit")?;

            self.bypass_whitespace()?;

            self.tag_expect_char("forsplit", |c| matches!(c, 'o'))?;
            self.tag_expect_char("forsplit", |c| matches!(c, 'n'))?;

            self.bypass_whitespace()?;

            let delimiter = self.parse_value("forsplit")?;

            self.bypass_whitespace()?;

            let mut from_idx = None;
            if self.tag_current_or_unexpected_eof("forsplit")? == 'f' {
                self.push_step()?;
                self.tag_expect_buffer_char("forsplit", |c| matches!(c, 'r'))?;
                self.tag_expect_buffer_char("forsplit", |c| matches!(c, 'o'))?;
                self.tag_expect_buffer_char("forsplit", |c| matches!(c, 'm'))?;

                self.bypass_whitespace()?;
                from_idx = Some(self.parse_value_as_number("forsplit")?);
                self.bypass_whitespace()?;
            }

            let mut to_idx = None;
            if self.tag_current_or_unexpected_eof("forsplit")? == 't' {
                self.push_step()?;
                self.tag_expect_buffer_char("forsplit", |c| matches!(c, 'o'))?;

                self.bypass_whitespace()?;
                to_idx = Some(self.parse_value_as_number("forsplit")?);
                self.bypass_whitespace()?;
            }

            let mut loop_variable = None;
            if self.tag_current_or_unexpected_eof("forsplit")? == 'a' {
                self.push_step()?;

                match self.tag_current_or_unexpected_eof("forsplit")? {
                    's' => {
                        self.push_step()?;

                        self.bypass_whitespace()?;

                        loop_variable = Some(self.parse_variable_name("forsplit")?);

                        self.bypass_whitespace()?;
                    },
                    _ => return self.tag_unexpected_char("forsplit"),
                }
            }

            let reversed = if self.tag_current_or_unexpected_eof("forsplit")? == 'r' {
                self.push_step()?;

                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'v'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'r'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'s'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'e'))?;
                self.tag_expect_buffer_char("forsplit", |c| c.eq(&'d'))?;

                self.output_mut().into_step()?.clear_buffer();
                self.bypass_whitespace()?;

                true
            }
            else {
                false
            };

            self.expect_end_of_tag("forsplit")?;

            let mut values = match str_value {
                Some(str_value) => match delimiter {
                    Some(delimiter) => if !delimiter.is_empty() {
                        str_value.split(&delimiter)
                            .map(|s| s.to_owned())
                            .collect::<Vec<String>>()
                    }
                    else {
                        str_value.chars()
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>()
                    },
                    None => str_value.chars().map(|c| c.to_string()).collect::<Vec<String>>(),
                },
                None => Vec::new(),
            };

            if from_idx.is_some() || to_idx.is_some() {
                let min = from_idx.unwrap_or(0_i64);
                let max = to_idx.unwrap_or(values.len() as i64);

                values = values.into_iter()
                    .enumerate()
                    .filter(|(i, _)| (i.to_owned() as i64) >= min)
                    .filter(|(i, _)| (i.to_owned() as i64) < max)
                    .map(|(_, v)| v)
                    .collect::<Vec<String>>();
            }

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndForsplit)
                .into_step()?;
            let else_content = match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndForsplit)
                        .into_step()?;
                    Some(else_content)
                },
                EndPosition::Forsplit => None,
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'forsplit' tag, '{pos:?}'"
                )))),
            };

            if !values.is_empty() {
                let last = values.len();

                if reversed {
                    values.reverse();
                }

                let path = self.input().into_step()?.path().to_owned();
                for (index, value) in values.into_iter().enumerate() {
                    self.context_mut().into_step()?.add_variable(&variable, &path, value.clone());

                    if let Some(loop_variable) = loop_variable.clone() {
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.index"), &path, index.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.size"), &path, last.to_string());
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.isfirst"), &path, if index == 0 { "1" } else { "0" });
                        self.context_mut().into_step()?
                            .add_variable(format!("{loop_variable}.islast"), &path, if index + 1 == last { "1" } else { "0" });
                    }

                    self.parse_limited(content.as_slice(), ParseUntil::EndForsplit).into_step()?;

                    self.context_mut().into_step()?.pop_variable(&variable);
                }
            }
            else if let Some(content) = else_content {
                self.parse_limited(content.as_slice(), ParseUntil::EndForsplit)
                    .into_step()?;
            }

            Ok(())
        }
    }

    fn parse_forspli(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_forsplit()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forspl(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_forspli()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_forsp(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_forspl()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_fors(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'p' => {
                self.push_step()?;
                self.parse_forsp()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_for(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_ford()
            },
            'e' => {
                self.push_step()?;
                self.parse_fore()
            },
            'f' => {
                self.push_step()?;
                self.parse_forf()
            },
            's' => {
                self.push_step()?;
                self.parse_fors()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_fo(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_for()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_fn(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("fn")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndFn)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Fn => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'fn' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let name = self.parse_variable_name("fn")?;
            let args = self.parse_function_args("fn")?;

            self.expect_end_of_tag("fn")?;

            let (content, end_pos,) = self.parse_bypassed(ParseUntil::EndFn).into_step()?;
            match end_pos {
                EndPosition::Fn => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'fn' tag, '{pos:?}'"
                )))),
            }

            let content_str = String::from_utf8(content)
                .into_internal(format!("Invalid utf-8 in function body of '{name}'"))
                .into_step()?;

            self.context_mut().into_step()?.add_function(name, args, content_str);

            Ok(())
        }
    }

    fn parse_f(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_fn()
            },
            'o' => {
                self.push_step()?;
                self.parse_fo()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_if(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_sequence("if", &['%', '}'])?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndIf)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndIf)
                        .into_step()?;
                    self.output_mut().into_step()?.write_bytes_to_buffer(else_content);
                },
                EndPosition::If => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'if' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let condition = IfParser::parse_result("if", self)?;

            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;
            self.expect_end_of_tag("if")?;

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndIf)
                .into_step()?;
            let else_content = match end_position {
                EndPosition::Else => {
                    let (else_content, ..) = self.parse_bypassed(ParseUntil::EndIf)
                        .into_step()?;
                    Some(else_content)
                },
                EndPosition::If => None,
                pos => return Err(Err(InternalError::new(format!("Invalid end position in 'if' tag, '{pos:?}'")))),
            };

            // parse if block
            if condition.as_evaluation() {
                self.parse_limited(content.as_slice(), ParseUntil::EndIf).into_step()?;
            }
            // parse else block
            else if let Some(else_content) = else_content {
                self.parse_limited(else_content.as_slice(), ParseUntil::EndIf).into_step()?;
            }

            Ok(())
        }
    }

    fn parse_include(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("include")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let path = self.parse_value_as_path("include")?
                .into_internal("The included path was None and this message needs improvement")
                .into_step()?;
            let file = OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(&path)
                .into_internal(format!("Failed to open file {path:?}"))
                .into_step()?;

            self.output_mut().into_step()?.flush_buffer_and_file(file)
                .into_step()?;

            self.expect_end_of_self_close_tag("include")?;

            Ok(())
        }
    }

    fn parse_includ(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_include()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_inclu(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_includ()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_incl(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'u' => {
                self.push_step()?;
                self.parse_inclu()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_inc(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_incl()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_in(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'c' => {
                self.push_step()?;
                self.parse_inc()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_i(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'f' => {
                self.push_step()?;
                self.parse_if()
            },
            'n' => {
                self.push_step()?;
                self.parse_in()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_length(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("length")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();
            let length = self.parse_value("length")?
                .unwrap_or_else(String::new)
                .len();
            self.expect_end_of_self_close_tag("length")?;

            self.output_mut().into_step()?.write_str(&length.to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_lengt(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_length()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_leng(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_lengt()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_len(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'g' => {
                self.push_step()?;
                self.parse_leng()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_le(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_len()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_l(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_le()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_mod(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("mod")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndMod)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Mod => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'mod' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("mod")?;

            self.expect_end_of_tag("mod")?;

            let content = self.parse_child(ParseUntil::EndMod).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            self.output_mut().into_step()?.write_str(&(value % content).to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_mo(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_mod()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_mul(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("mul")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndMul)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Mul => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'mul' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("mul")?;

            self.expect_end_of_tag("mul")?;

            let content = self.parse_child(ParseUntil::EndMul).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            self.output_mut().into_step()?.write_str(&(value * content).to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_mu(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_mul()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_m(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'o' => {
                self.push_step()?;
                self.parse_mo()
            },
            'u' => {
                self.push_step()?;
                self.parse_mu()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_nth(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("nth")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndNth)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Nth => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'nth' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            let alias = self.parse_variable_name("nth")?;

            self.expect_end_of_tag("nth")?;

            let output = self.parse_child(ParseUntil::EndNth).into_step()?;
            let values = self.context().into_step()?.values(&alias);

            let trimmed = output.trim();

            let was_neg_zero;
            let idx = if trimmed == "-0" {
                was_neg_zero = true;
                0
            }
            else {
                was_neg_zero = false;
                output.trim()
                    .parse::<i64>()
                    .map_err(|_| InternalError::new(format!(
                        "Content was not an integer:\n{output}"
                    )))
                    .into_step()?
            };

            let idx_to_take = if was_neg_zero || idx < 0_i64 {
                values.as_ref().map_or(0_usize, |vs| vs.len()
                    .saturating_sub(1)
                    .saturating_sub((0 - idx) as usize)
                )
            }
            else {
                idx as usize
            };

            let value = values.and_then(
                |vs| vs.into_iter().nth(idx_to_take).map(|v| v.to_owned())
            );

            self.output_mut().into_step()?.write_str(&value.unwrap_or(String::new()));
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_nt(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_nth()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_n(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_nt()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_path(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.buffer_all_until_end_of_self_closing_tag("path")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let path = self.parse_value("path")?
                .into_internal("Cannot canonicalize a path from an empty value")
                .into_step()?;

            self.bypass_whitespace()?;

            let mut reldir = if self.tag_current_or_unexpected_eof("path")? == 'i' {
                self.push_step()?;

                self.tag_expect_char("path", |c| matches!(c, 'n'))?;
                self.bypass_whitespace()?;

                self.output_mut().into_step()?.clear_buffer();

                self.parse_value_as_path("path")?
                    .into_internal("Cannot create an absolute path from a None path")
                    .into_step()?
            }
            else {
                let mut input_path = self.input().into_step()?.path().to_owned();
                input_path.pop();
                input_path
            };

            reldir.push(path);

            let dir = canonicalize(&reldir)
                .into_internal(format!("Failed to canonicalize relative path {reldir:?}"))
                .into_step()?;

            self.bypass_whitespace()?;

            self.output_mut().into_step()?.clear_buffer();
            self.output_mut().into_step()?.write_str(dir.to_str().unwrap_or(""));
            self.output_mut().into_step()?.flush_buffer_to_content();

            self.expect_end_of_self_close_tag("path")?;

            Ok(())
        }
    }

    fn parse_pat(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_path()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_pa(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_pat()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_pow(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("pow")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndPow)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Pow => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'pow' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("pow")?;

            self.expect_end_of_tag("pow")?;

            let content = self.parse_child(ParseUntil::EndPow).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            let val_u32: u32 = value.try_into()
                .into_internal("Value was too large for 'pow' operations")
                .into_step()?;
            let con_u32: u32 = content.try_into()
                .into_internal("Content produced too large a number for 'pow' operations")
                .into_step()?;
            let pow_u32 = val_u32.checked_pow(con_u32)
                .into_internal("Value produced by 'pow' operation caused an overflow")
                .into_step()?;

            self.output_mut().into_step()?.write_str(&pow_u32.to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_po(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'w' => {
                self.push_step()?;
                self.parse_pow()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_p(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_pa()
            },
            'o' => {
                self.push_step()?;
                self.parse_po()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_set(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("set")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndSet)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Set => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'set' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let variable = self.parse_variable_name("set")?;

            self.expect_end_of_tag("set")?;

            let content = self.parse_child(ParseUntil::EndSet).into_step()?;
            let path = self.input().into_step()?.path().to_owned();

            self.context_mut().into_step()?.add_variable(variable, path, content);

            Ok(())
        }
    }

    fn parse_se(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_set()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_sub(&mut self) -> StepResult<()> {
        if self.bypass() {
            if !self.buffer_whitespace_enforce_one()? {
                return self.unexpected_tag();
            }

            self.buffer_all_until_end_of_tag("sub")?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            let (content, end_position) = self.parse_bypassed(ParseUntil::EndSub)
                .into_step()?;
            self.output_mut().into_step()?.write_bytes_to_buffer(content);

            match end_position {
                EndPosition::Sub => {},
                pos => return Err(Err(InternalError::new(format!(
                    "Invalid end position in 'sub' tag, '{pos:?}'"
                )))),
            };

            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            if !self.bypass_whitespace_enforce_one()? {
                return self.unknown_tag();
            }

            self.output_mut().into_step()?.clear_buffer();

            let value = self.parse_value_as_number("sub")?;

            self.expect_end_of_tag("sub")?;

            let content = self.parse_child(ParseUntil::EndSub).into_step()?
                .trim()
                .parse::<i64>()
                .into_internal("Failed to parse content as a number")
                .into_step()?;

            self.output_mut().into_step()?.write_str(&(value - content).to_string());
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
    }

    fn parse_su(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'b' => {
                self.push_step()?;
                self.parse_sub()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_s(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_se()
            },
            'u' => {
                self.push_step()?;
                self.parse_su()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_add(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndAdd => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("add")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Add);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndAdd => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("add")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_ad(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_end_add()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_a(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_end_ad()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_div(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndDiv => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("mul")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Div);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndDiv => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("mul")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_di(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'v' => {
                self.push_step()?;
                self.parse_end_div()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_d(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_end_di()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_fordir(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndFordir => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("fordir")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Fordir);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndFordir => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("fordir")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_fordi(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_end_fordir()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_ford(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_end_fordi()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_foreach(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForeach => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("foreach")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Foreach);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForeach => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("foreach")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_foreac(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_end_foreach()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forea(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'c' => {
                self.push_step()?;
                self.parse_end_foreac()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_fore(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_end_forea()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forfile(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForfile => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("forfile")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Forfile);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForfile => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("forfile")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_forfil(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_end_forfile()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forfi(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_end_forfil()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forf(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_end_forfi()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forsplit(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForsplit => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("forsplit")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Forsplit);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndForsplit => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("forsplit")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_forspli(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_end_forsplit()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forspl(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'i' => {
                self.push_step()?;
                self.parse_end_forspli()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_forsp(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_end_forspl()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_fors(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'p' => {
                self.push_step()?;
                self.parse_end_forsp()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_for(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_end_ford()
            },
            'e' => {
                self.push_step()?;
                self.parse_end_fore()
            },
            'f' => {
                self.push_step()?;
                self.parse_end_forf()
            },
            's' => {
                self.push_step()?;
                self.parse_end_fors()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_fo(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'r' => {
                self.push_step()?;
                self.parse_end_for()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_fn(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndFn => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("fn")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Fn);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndFn => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("fn")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_f(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'n' => {
                self.push_step()?;
                self.parse_end_fn()
            },
            'o' => {
                self.push_step()?;
                self.parse_end_fo()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_if(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndIf => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("if")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::If);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndIf => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("if")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_i(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'f' => {
                self.push_step()?;
                self.parse_end_if()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_mod(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndMod => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }
            self.expect_end_of_end_tag_buffer("mod")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Mod);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndMod => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("mod")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_mo(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'd' => {
                self.push_step()?;
                self.parse_end_mod()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_mul(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndMul => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("mul")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Mul);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndMul => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("mul")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_mu(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'l' => {
                self.push_step()?;
                self.parse_end_mul()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_m(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'o' => {
                self.push_step()?;
                self.parse_end_mo()
            },
            'u' => {
                self.push_step()?;
                self.parse_end_mu()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_nth(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndNth => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }
            self.expect_end_of_end_tag_buffer("nth")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Nth);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndNth => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("nth")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_nt(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'h' => {
                self.push_step()?;
                self.parse_end_nth()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_n(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_end_nt()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_pow(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndPow => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("pow")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Pow);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndPow => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("pow")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_po(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'w' => {
                self.push_step()?;
                self.parse_end_pow()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_p(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'o' => {
                self.push_step()?;
                self.parse_end_po()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_set(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndSet => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("set")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Set);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndSet => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("set")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_se(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            't' => {
                self.push_step()?;
                self.parse_end_set()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_sub(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndSub => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag_buffer("sub")?;
            self.output_mut().into_step()?.flush_buffer_to_content();
            self.set_end_position(EndPosition::Sub);

            Err(Ok(FlowControl::Break))
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            match self.parse_until() {
                ParseUntil::EndSub => {},
                _ => {
                    return self.unexpected_end_tag();
                },
            }

            self.expect_end_of_end_tag("sub")?;

            Err(Ok(FlowControl::Break))
        }
    }

    fn parse_end_su(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'b' => {
                self.push_step()?;
                self.parse_end_sub()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end_s(&mut self) -> StepResult<()> {
        match self.current_or_unexpected_eof_in_tag()? {
            'e' => {
                self.push_step()?;
                self.parse_end_se()
            },
            'u' => {
                self.push_step()?;
                self.parse_end_su()
            },
            _ => self.unexpected_tag(),
        }
    }

    fn parse_end(&mut self) -> StepResult<()> {
        if let ParseUntil::Eof = self.parse_until() {
            return self.unexpected_end_tag();
        }

        if !self.bypass() {
            // clear buffer, we got an end tag
            self.output_mut().into_step()?.clear_buffer();
        }

        // bypass all starting whitespace
        self.bypass_whitespace()?;

        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_end_a()
            },
            'd' => {
                self.push_step()?;
                self.parse_end_d()
            },
            'f' => {
                self.push_step()?;
                self.parse_end_f()
            },
            'i' => {
                self.push_step()?;
                self.parse_end_i()
            },
            'm' => {
                self.push_step()?;
                self.parse_end_m()
            },
            'n' => {
                self.push_step()?;
                self.parse_end_n()
            },
            'p' => {
                self.push_step()?;
                self.parse_end_p()
            },
            's' => {
                self.push_step()?;
                self.parse_end_s()
            },
            _ => self.unknown_end_tag(),
        }
    }

    fn parse_tag(&mut self) -> StepResult<()> {
        if !self.bypass() {
            // clear buffer, we got a tag
            self.output_mut().into_step()?.clear_buffer();
        }

        // bypass all starting whitespace
        self.buffer_whitespace()?;

        match self.current_or_unexpected_eof_in_tag()? {
            'a' => {
                self.push_step()?;
                self.parse_a()
            },
            'b' => {
                self.push_step()?;
                self.parse_b()
            },
            'c' => {
                self.push_step()?;
                self.parse_c()
            },
            'd' => {
                self.push_step()?;
                self.parse_d()
            },
            'e' => {
                self.push_step()?;
                self.parse_e()
            },
            'f' => {
                self.push_step()?;
                self.parse_f()
            },
            'i' => {
                self.push_step()?;
                self.parse_i()
            },
            'l' => {
                self.push_step()?;
                self.parse_l()
            },
            'm' => {
                self.push_step()?;
                self.parse_m()
            },
            'n' => {
                self.push_step()?;
                self.parse_n()
            },
            'p' => {
                self.push_step()?;
                self.parse_p()
            },
            's' => {
                self.push_step()?;
                self.parse_s()
            },
            '/' => {
                self.push_step()?;
                self.parse_end()
            },
            _ => self.unknown_tag(),
        }
    }

    fn parse_comment(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.buffer_all_until_sequence("comment", &['#', '}'])
        }
        else {
            // clear buffer, we got a comment
            self.output_mut().into_step()?.clear_buffer();

            loop {
                self.bypass_all_until(|c| matches!(c, '#'))?;

                self.input_mut().into_step()?.step().into_step()?;
                let c = match self.current()? {
                    Some(c) => c,
                    None => return Err(Err(InternalError::new("Unclosed comment"))),
                };

                if c == '}' {
                    self.input_mut().into_step()?.step().into_step()?;
                    break;
                }
            }

            Ok(())
        }
    }

    fn parse_output(&mut self) -> StepResult<()> {
        if self.bypass() {
            self.buffer_whitespace()?;
            self.buffer_all_until_sequence("output", &['}', '}'])?;
            self.output_mut().into_step()?.flush_buffer_to_content();

            Ok(())
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;

            let alias = self.parse_variable_name("output")?;

            self.bypass_whitespace()?;

            match self.current_or_unexpected_eof_in_tag()? {
                '(' => {
                    let function = self.context().into_step()?.function(&alias)
                        .into_internal(format!("Function '{alias}' never defined"))
                        .into_step()?
                        .to_owned();

                    let mut args = self.parse_function_arg_values("exec")?
                        .into_iter();
                    let mut ctx = self.context().into_step()?.to_owned();

                    for named in function.args().iter() {
                        ctx.remove_variable(named);

                        if let Some(Some(arg)) = args.next() {
                            ctx.add_variable(named, self.input().into_step()?
                                .path(), &arg);
                        }
                    }

                    // make sure we write all buffered content before spawning the sealed
                    // parser
                    self.output_mut().into_step()?.flush_buffer_to_content();
                    self.parse_limited_sealed(ctx, function.as_bytes(), ParseUntil::EndFn)
                        .into_step()?;
                },
                _ => {
                    let value = self.context().into_step()?.value(&alias);

                    let output = value.map_or(String::new(), |v| v.to_owned());
                    self.output_mut().into_step()?.write_str(&output);
                    self.output_mut().into_step()?.flush_buffer_to_content();
                },
            }

            self.bypass_whitespace()?;
            self.tag_expect_char("output", |c| matches!(c, '}'))?;
            self.tag_expect_char("output", |c| matches!(c, '}'))?;

            Ok(())
        }
    }

    fn parse_bracket(&mut self) -> StepResult<()> {
        let c = match self.current()? {
            Some(c) => c,
            None => return Err(Ok(FlowControl::Continue)),
        };

        match c {
            '{' => {
                self.push_step()?;
                self.parse_output()
            },
            '%' => {
                self.push_step()?;
                self.parse_tag()
            },
            '#' => {
                self.push_step()?;
                self.parse_comment()
            },
            _ => {
                self.output_mut().into_step()?.flush_buffer_to_content();
                Ok(())
            },
        }
    }

    fn handle_escape(&mut self) -> StepResult<()> {
        if self.bypass() {
            Ok(())
        }
        else {
            self.output_mut().into_step()?.clear_buffer();
            self.bypass_whitespace()?;
            Ok(())
        }
    }

    fn parse_internal(&mut self) -> InternalResult<()> {
        loop {
            if self.input()?.is_end() {
                break;
            }

            let c = flow_internal!(self.current_or_continue());
            let res = match c {
                '{' => {
                    self.push_step_internal().or_else_upgrade(self)?;
                    self.parse_bracket().or_else_upgrade(self)
                },
                '\\' => {
                    self.push_step_internal().or_else_upgrade(self)?;
                    self.handle_escape().or_else_upgrade(self)
                },
                _ => {
                    match self.output_mut() {
                        Ok(output) => output.write_char(c),
                        Err(e) => Err(e).or_else_upgrade(self)?,
                    }

                    match self.input_mut() {
                        Ok(input) => match input.step() {
                            Ok(_) => {},
                            Err(e) => Err(e).or_else_upgrade_from_input(input)?,
                        },
                        Err(e) => Err(e).or_else_upgrade(self)?,
                    }

                    match self.output_mut() {
                        Ok(output) => output.flush_buffer_to_content(),
                        Err(e) => Err(e).or_else_upgrade(self)?,
                    }

                    continue;
                },
            };

            flow_internal!(res);

            self.set_end_position(EndPosition::Eof);
        }

        if let Some(extend) = self.extend.take() {
            self.parse_extend(extend)?;
        }
        else {
            self.output_mut()?.flush_buffer_to_content();
        }

        Ok(())
    }

    pub(crate) fn parse(&mut self) -> InternalResult<()> {
        self.parse_internal()?;
        self.write()
    }

    fn write(&mut self) -> InternalResult<()> {
        if let Some(output) = self.output.as_mut() {
            output.write_content_to_destination()?;
        }

        Ok(())
    }
}
