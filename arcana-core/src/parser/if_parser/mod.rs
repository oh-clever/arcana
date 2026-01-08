use {
    crate::{
        context::{
            Context,
            Variable,
        },
        error::{
            IntoInternal,
            InternalError,
            InternalResult,
        },
        input::Input,
        output::Output,
        parser::{
            Parser,
            ParseUntil,
            TemplateParser,
            steps::*,
        },
    },
    std::{
        fmt::Debug,
        io::{ Read, Write, },
    },
};

#[derive(Debug)]
enum Join {
    And,
    Or,
}

#[derive(Debug)]
pub(crate) struct Condition {
    evaluation: bool,
    join: Option<Join>,
}

impl From<bool> for Condition {
    fn from(input: bool) -> Self {
        Self { evaluation: input, join: None, }
    }
}

impl Condition {
    fn negate(&mut self) {
        self.evaluation = !self.evaluation;
    }

    fn requires_next(&self) -> InternalResult<bool> {
        match self.join {
            Some(Join::And) => Ok(self.evaluation),
            Some(Join::Or) => Ok(!self.evaluation),
            None => Err(InternalError::new("Checked if condition requires next and there is no join")),
        }
    }

    fn set_join(&mut self, join: Join) {
        self.join = Some(join);
    }

    pub(crate) fn as_evaluation(&self) -> bool {
        self.evaluation
    }

    fn try_num<F, H>(a: Option<String>, b: Option<String>, num_cmp: F, str_cmp: H) -> Self
    where
        F: Fn(&i64, &i64) -> bool,
        H: Fn(&Option<String>, &Option<String>) -> bool,
    {
        if a.is_none() {
            return Self::from(str_cmp(&a, &b));
        }

        let ar = a.as_ref().unwrap();
        let ir = ar.parse::<i64>();
        if ir.is_err() {
            return Self::from(str_cmp(&a, &b));
        }

        if b.is_none() {
            return Self::from(str_cmp(&a, &b));
        }

        let br = b.as_ref().unwrap();
        let jr = br.parse::<i64>();
        if jr.is_err() {
            return Self::from(str_cmp(&a, &b));
        }

        let i = ir.unwrap();
        let j = jr.unwrap();

        Self::from(num_cmp(&i, &j))
    }

    pub(crate) fn gt(a: Option<String>, b: Option<String>) -> Self {
        Self::try_num(a, b, i64::gt, Option::<String>::gt)
    }

    pub(crate) fn ge(a: Option<String>, b: Option<String>) -> Self {
        Self::try_num(a, b, i64::ge, Option::<String>::ge)
    }

    pub(crate) fn lt(a: Option<String>, b: Option<String>) -> Self {
        Self::try_num(a, b, i64::lt, Option::<String>::lt)
    }

    pub(crate) fn le(a: Option<String>, b: Option<String>) -> Self {
        Self::try_num(a, b, i64::le, Option::<String>::le)
    }
}

#[derive(Debug)]
pub(crate) struct IfParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    context: Option<Context>,
    condition: Option<Condition>,
    input: Option<Input<R>>,
    output: Option<Output<W>>,
    parse_until: ParseUntil,
    bypass: Option<bool>,
    tagname: String,
}

impl<R, W> Parser<R, W> for IfParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    fn context(&self) -> InternalResult<&Context> {
        self.context.as_ref().into_internal("Context is None")
    }

    fn context_mut(&mut self) -> InternalResult<&mut Context> {
        self.context.as_mut().into_internal("Input is None")
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

impl<R, W> IfParser<R, W>
where
    R: Read + Debug,
    W: Write + Debug,
{
    fn new<S: AsRef<str>>(
        tagname: S, context: Context, input: Input<R>,
        output: Output<W>, parse_until: ParseUntil,
        bypass: Option<bool>,
    ) -> Self {
        Self {
            tagname: tagname.as_ref().to_owned(),
            context: Some(context),
            condition: None,
            input: Some(input),
            output: Some(output),
            parse_until,
            bypass,
        }
    }

    fn condition_mut(&mut self) -> InternalResult<&mut Condition> {
        self.condition.as_mut().into_internal("Condition was None")
    }

    fn parse(&mut self) -> StepResult<Condition> {
        loop {
            let mut c = match self.current()? {
                Some(c) => c,
                None => return self.tag_unexpected_eof(&self.tagname),
            };

            let tagname = self.tagname.to_owned();

            let mut negate = false;
            while matches!(c, '!') {
                self.input_mut().into_step()?.step().into_step()?;

                c = match self.current()? {
                    Some(c) => c,
                    None => return self.tag_unexpected_eof(&self.tagname),
                };

                negate = !negate;
            }

            match c {
                '(' => {
                    self.input_mut().into_step()?.step().into_step()?;

                    let bypass = match self.bypass.as_ref() {
                        // this was bypassed, so all children should be bypassed
                        Some(b) => Some(*b),
                        None => match self.condition.as_ref() {
                            Some(c) => if !c.requires_next().into_step()? {
                                // next is not required, rely on this evaluation
                                Some(c.evaluation)
                            }
                            else {
                                // next is required, do not bypass
                                None
                            },
                            // condition may correctly be not set yet
                            None => None,
                        },
                    };

                    let condition = Self::parse_condition(
                        &tagname,
                        self,
                        ParseUntil::ConditionEnd,
                        bypass,
                    )?;

                    self.condition = Some(condition);
                },
                ')' => {
                    self.input_mut().into_step()?.step().into_step()?;

                    match &self.parse_until {
                        ParseUntil::ConditionEnd => match self.condition.as_ref() {
                            Some(c) => if c.join.is_some() {
                                return self.tag_unexpected_char_expected(&self.tagname, ")");
                            }
                            else {
                                break;
                            },
                            None => {
                                return Err(Err(InternalError::new(concat!(
                                    "If tag fell into an unexpected state: ",
                                    "a valid condition end was found but no ",
                                    "condition was set.",
                                ))));
                            },
                        },
                        _ => return self.tag_unexpected_char(&self.tagname),
                    }
                },
                _ => {
                    self.bypass_whitespace()?;

                    let left_value = self.parse_value(&tagname)?;

                    self.bypass_whitespace()?;

                    match self.tag_current_or_unexpected_eof(&tagname)? {
                        '=' => {
                            self.input_mut().into_step()?.step().into_step()?;

                            match self.tag_current_or_unexpected_eof(&tagname)? {
                                '=' => {
                                    self.input_mut().into_step()?.step().into_step()?;
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;
                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::from(left_value == right_value),
                                    });
                                },
                                _ => return self.tag_unexpected_char_expected(&tagname, "=")?,
                            }
                        },
                        '!' => {
                            self.input_mut().into_step()?.step().into_step()?;

                            match self.tag_current_or_unexpected_eof(&tagname)? {
                                '=' => {
                                    self.input_mut().into_step()?.step().into_step()?;
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;
                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::from(left_value != right_value),
                                    });
                                },
                                _ => return self.tag_unexpected_char_expected(&tagname, "="),
                            }
                        },
                        '>' => {
                            self.input_mut().into_step()?.step().into_step()?;

                            match self.tag_current_or_unexpected_eof(&tagname)? {
                                '=' => {
                                    self.input_mut().into_step()?.step().into_step()?;
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;
                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::ge(left_value, right_value),
                                    });
                                },
                                _ => {
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;

                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::gt(left_value, right_value),
                                    });
                                },
                            }
                        },
                        '<' => {
                            self.input_mut().into_step()?.step().into_step()?;

                            match self.tag_current_or_unexpected_eof(&tagname)? {
                                '=' => {
                                    self.input_mut().into_step()?.step().into_step()?;
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;
                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::le(left_value, right_value),
                                    });
                                },
                                _ => {
                                    self.bypass_whitespace()?;

                                    let right_value = self.parse_value(&tagname)?;
                                    self.condition = Some(match self.bypass.as_ref() {
                                        Some(b) => Condition::from(*b),
                                        None => Condition::lt(left_value, right_value),
                                    });
                                },
                            }
                        },
                        // truthy
                        _ => {
                            self.condition = Some(match self.bypass.as_ref() {
                                Some(b) => Condition::from(*b),
                                None => Condition::from(Variable::value_is_truthy(left_value.clone())),
                            });
                        },
                    }
                },
            }

            self.bypass_whitespace()?;
            match self.tag_current_or_unexpected_eof(&tagname)? {
                '&' => {
                    self.input_mut().into_step()?.step().into_step()?;

                    match self.tag_current_or_unexpected_eof(&tagname)? {
                        '&' => {
                            self.input_mut().into_step()?.step().into_step()?;
                            self.condition_mut().into_step()?.set_join(Join::And);
                        },
                        _ => return self.tag_unexpected_char(&self.tagname),
                    }
                },
                '|' => {
                    self.input_mut().into_step()?.step().into_step()?;

                    match self.tag_current_or_unexpected_eof(&tagname)? {
                        '|' => {
                            self.input_mut().into_step()?.step().into_step()?;
                            self.condition_mut().into_step()?.set_join(Join::Or);
                        },
                        _ => return self.tag_unexpected_char(&self.tagname),
                    }
                },
                // end of conditions
                _ => {},
            }

            match self.condition.as_mut() {
                Some(c) => {
                    if self.bypass.is_none() && negate {
                        c.negate();
                    }

                    match &self.parse_until {
                        // still need to find the ')' character
                        ParseUntil::ConditionEnd => {
                            if c.join.is_some() && !c.requires_next().into_step()? {
                                self.bypass = Some(c.evaluation);
                            }

                            self.bypass_whitespace()?;
                            continue;
                        },
                        ParseUntil::Eot => {
                            // joined with "&&" or "||" and another loop is
                            // required
                            if c.join.is_some() {
                                if !c.requires_next().into_step()? {
                                    self.bypass = Some(c.evaluation);
                                }

                                self.bypass_whitespace()?;
                                continue;
                            }
                            // seeking end of tag and no join, we should be done
                            else {
                                self.bypass_whitespace()?;
                                break;
                            }
                        },
                        p => {
                            return Err(Err(InternalError::new(format!(
                                "'{:?}' is not a valid parse until for '{}' tags",
                                p, self.tagname.to_owned()
                            ))));
                        },
                    }
                },
                None => {
                    return Err(Err(InternalError::new(format!(
                        "Condition of '{}' was never set, ended prematurely",
                        self.tagname.to_owned(),
                    ))));
                },
            }
        }

        self.condition.take()
            .into_internal(format!("Condition of '{}' was never set, ended prematurely", self.tagname.to_owned()))
            .into_step()
    }

    fn parse_condition<S: AsRef<str>, P>(tagname: S, parser: &mut P, parse_until: ParseUntil, bypass: Option<bool>) -> StepResult<Condition>
    where
        P: Parser<R, W>,
    {
        let mut ifp = Self::new(
            tagname,
            parser.take_context().into_step()?,
            parser.take_input().into_step()?, parser.take_output().into_step()?,
            parse_until, bypass,
        );

        let result = ifp.parse()?;

        let context = ifp.take_context().into_step()?;
        let input = ifp.take_input().into_step()?;
        let output = ifp.take_output().into_step()?;
        drop(ifp);

        parser.give_context(Some(context));
        parser.give_input(Some(input));
        parser.give_output(Some(output));

        Ok(result)
    }

    pub(crate) fn parse_result<S: AsRef<str>>(tagname: S, parser: &mut TemplateParser<R, W>) -> StepResult<Condition> {
        Self::parse_condition(tagname, parser, ParseUntil::Eot, None)
    }
}
