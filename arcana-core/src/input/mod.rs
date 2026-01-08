#[cfg(test)]
mod test;

use {
    crate::error::{ IntoInternal, InternalResult, },
    std::{
        fmt::Debug,
        fs::{ OpenOptions, File, },
        io::{ BufRead, BufReader, Read, },
        path::{ Path, PathBuf },
    },
};

#[derive(Debug)]
struct InputString {
    value: String,
    start_index: usize,
    end_index: usize,
    current: Option<char>,
}

impl InputString {
    fn current(&self) -> Option<&char> {
        self.current.as_ref()
    }

    pub(crate) fn line(&self) -> String {
        self.value.to_owned()
    }

    fn step(&mut self) {
        self.start_index = self.end_index;

        let mut end = self.end_index + 1;
        while !self.value.is_char_boundary(end) && end <= self.value.len() {
            end += 1;
        }

        self.end_index = end;
        if self.end_index > self.value.len() {
            self.current = None;
        }
        else {
            self.current = self.value[self.start_index..self.end_index].chars().nth(0);
        }
    }

    fn is_end(&self) -> bool {
        self.current.is_none()
    }
}

impl From<String> for InputString {
    fn from(input: String) -> Self {
        Self {
            value: input,
            start_index: 0,
            end_index: 0,
            current: None,
        }
    }
}

#[derive(Debug)]
pub struct Input<R>
where
    R: Read + Debug
{
    path: PathBuf,
    source: BufReader<R>,
    read: Option<InputString>,
    line: usize,
    index: usize,
}

pub trait TryIntoInput<R>
where
    R: Read + Debug,
{
    fn try_into_input(self) -> InternalResult<Input<R>>;
}

impl<R> TryIntoInput<R> for Input<R>
where
    R: Read + Debug
{
    fn try_into_input(self) -> InternalResult<Self> {
        Ok(self)
    }
}

impl<R> TryIntoInput<R> for R
where
    R: Read + Debug
{
    fn try_into_input(self) -> InternalResult<Input<R>> {
        let br = BufReader::new(self);

        let mut input = Input::<R> {
            path: PathBuf::new(),
            source: br,
            read: None,
            line: 0,
            index: 0,
        };

        input.step()?;

        Ok(input)
    }
}

impl TryIntoInput<File> for &Path {
    fn try_into_input(self) -> InternalResult<Input<File>> {
        let file = OpenOptions::new()
            .read(true)
            .create(false)
            .write(false)
            .open(self)
            .into_internal(format!("Failed to open file at {self:?}"))?;

        let mut input = Input::<File> {
            path: self.to_owned(),
            source: BufReader::new(file),
            read: None,
            line: 0,
            index: 0,
        };

        input.step()?;

        Ok(input)
    }
}

impl TryIntoInput<File> for PathBuf {
    fn try_into_input(self) -> InternalResult<Input<File>> {
        let path: &Path = self.as_ref();
        path.try_into_input()
    }
}

impl<R> Input<R>
where
    R: Read + Debug,
{
    pub(crate) fn join_path<P>(&self, p: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        let path = p.as_ref();
        if path.is_absolute() {
            return path.into();
        }

        let mut new_path = if !self.path.is_dir() {
            let mut new_path = self.path.clone();
            new_path.pop();
            new_path
        }
        else {
            self.path.clone()
        };

        new_path.push(path);

        new_path
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn line_no(&self) -> usize {
        self.line
    }

    pub(crate) fn line(&self) -> String {
        self.read.as_ref().map_or(String::new(), |r| r.line())
    }

    pub(crate) fn current(&self) -> Option<&char> {
        self.read.as_ref().and_then(InputString::current)
    }

    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }

    pub(crate) fn set_path<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        self.path = path.as_ref().into();
    }

    pub(crate) fn step(&mut self) -> InternalResult<()> {
        if let Some(is) = self.read.as_mut() {
            is.step();
            self.index += 1;
            if !is.is_end() {
                return Ok(());
            }
        }

        let mut content = String::new();
        self.source.read_line(&mut content).into_internal("Failed to read line")?;

        if content.is_empty() {
            self.read = None;
            return Ok(());
        }

        self.index = 0;
        self.line += 1;
        let mut input_string = InputString::from(content);
        input_string.step();
        self.read = Some(input_string);

        Ok(())
    }

    pub(crate) fn is_end(&self) -> bool {
        self.read.as_ref().is_none_or(|r| r.is_end())
    }
}

impl<'string> TryIntoInput<&'string [u8]> for &'string str {
    fn try_into_input(self) -> InternalResult<Input<&'string [u8]>> {
        self.as_bytes().try_into_input()
    }
}
