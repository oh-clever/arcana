use {
    crate::{
        input::Input,
        parser::Parser,
    },
    std::{ error, fmt, io, path, result, },
};

#[derive(Debug)]
pub struct ErrorPosition {
    file: path::PathBuf,
    index: usize,
    line: usize,
    hint: String,
}

impl ErrorPosition {
    fn new(file: path::PathBuf, index: usize, line: usize, hint: String) -> Self {
        Self { file, index, line, hint, }
    }
}

impl<'input, R> From<&'input Input<R>> for ErrorPosition
where
    R: io::Read + fmt::Debug
{
    fn from(input: &'input Input<R>) -> Self {
        Self::new(input.path().to_owned(), input.index(), input.line_no(), input.line())
    }
}

#[derive(Debug)]
pub struct InternalError {
    position: Option<ErrorPosition>,
    message: String,
}

impl InternalError {
    pub(crate) fn new<S>(msg: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            position: None,
            message: msg.as_ref().to_owned(),
        }
    }

    pub(crate) fn upgrade_from_input<R>(&mut self, input: &Input<R>)
    where
        R: io::Read + fmt::Debug,
    {
        if self.position.is_some() {
            return;
        }

        self.position = Some(input.into());
    }

    pub(crate) fn upgrade<R, W, P>(&mut self, parser: &P)
    where
        R: io::Read + fmt::Debug,
        W: io::Write + fmt::Debug,
        P: Parser<R, W>,
    {
        if self.position.is_some() {
            return;
        }

        self.position = parser.input_opt().map(|i| i.into());
    }

    #[cfg(test)]
    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

pub type InternalResult<T> = result::Result<T, InternalError>;

pub(crate) trait IntoInternal<T> {
    fn into_internal<S>(self, message: S) -> InternalResult<T>
    where
        S: AsRef<str>;
}

impl<T, E> IntoInternal<T> for result::Result<T, E>
where
    E: error::Error
{
    fn into_internal<S>(self, message: S) -> InternalResult<T>
    where
        S: AsRef<str>,
    {
        self.map_err(|_| InternalError::new(message))
    }
}

impl<T> IntoInternal<T> for Option<T> {
    fn into_internal<S>(self, message: S) -> InternalResult<T>
    where
        S: AsRef<str>,
    {
        self.map(|v| Ok(v)).unwrap_or(Err(InternalError::new(message)))
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        if let Some(position) = self.position.as_ref() {
            fmtr.write_fmt(format_args!(
                "Error '{}'\nIn '{:?}'\nOccured on line {}:{}\nNear '{}'",
                self.message,
                position.file,
                position.line,
                position.index,
                position.hint,
            ))
        }
        else {
            fmtr.write_fmt(format_args!(
                "Error '{}'\nPosition unknown (internal failure)",
                self.message,
            ))
        }
    }
}

impl error::Error for InternalError {}

pub(crate) trait OrElseUpgrade<T> {
    type Output;

    fn or_else_upgrade<R, W, P>(self, parser: &P) -> Self::Output
    where
        R: io::Read + fmt::Debug,
        W: io::Write + fmt::Debug,
        P: Parser<R, W>;

    fn or_else_upgrade_from_input<R>(self, input: &Input<R>) -> Self::Output
    where
        R: io::Read + fmt::Debug;
}

impl<T> OrElseUpgrade<T> for InternalResult<T> {
    type Output = Self;

    fn or_else_upgrade<R, W, P>(self, parser: &P) -> Self
    where
        R: io::Read + fmt::Debug,
        W: io::Write + fmt::Debug,
        P: Parser<R, W>,
    {
        self.map_err(|mut e| {
            e.upgrade(parser);
            e
        })
    }

    fn or_else_upgrade_from_input<R>(self, input: &Input<R>) -> Self::Output
    where
        R: io::Read + fmt::Debug
    {
        self.map_err(|mut e| {
            e.upgrade_from_input(input);
            e
        })
    }
}
