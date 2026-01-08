use {
    crate::error::{
        IntoInternal,
        InternalResult,
    },
    std::{
        fmt::Debug,
        fs::File,
        io::{ BufRead, BufReader, BufWriter, Write },
    },
};

#[derive(Debug)]
pub struct Output<W>
where
    W: Write + Debug
{
    buffer: Vec<u8>,
    content: Vec<u8>,
    destination: BufWriter<W>,
}

impl<W> Output<W>
where
    W: Write + Debug,
{
    fn new(writer: W) -> Self {
        Self {
            buffer: Vec::new(),
            content: Vec::new(),
            destination: BufWriter::new(writer),
        }
    }

    pub(crate) fn write_bytes_to_buffer(&mut self, mut bytes: Vec<u8>) {
        self.buffer.append(&mut bytes);
    }

    pub(crate) fn write_char(&mut self, c: char) {
        self.buffer.push(c as u8);
    }

    pub(crate) fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub(crate) fn flush_buffer_to_content(&mut self) {
        self.content.append(&mut self.buffer);
    }

    pub(crate) fn flush_buffer_and_file(&mut self, file: File) -> InternalResult<()> {
        self.flush_buffer_to_content();

        let br = BufReader::new(file);
        let mut first_line = true;
        let lines = br.lines();
        for line in lines {
            let line = line.into_internal("Failed to read next line")?;
            if !first_line {
                self.content.write_all(format!("\n{line}").as_bytes())
                    .into_internal("Failed to write line from file")?;
            }
            else {
                self.content.write_all(line.as_bytes())
                    .into_internal("Failed to write line from file")?;
                first_line = false;
            }
        }

        Ok(())
    }

    pub(crate) fn write_content_to_destination(&mut self) -> InternalResult<()> {
        self.destination.write_all(&self.content)
            .into_internal("Failed to write content to destination")?;
        self.content.clear();
        Ok(())
    }

    pub(crate) fn take_content(&mut self) -> Vec<u8> {
        let mut content = Vec::new();
        content.append(&mut self.content);
        content
    }

    pub(crate) fn take_buffer(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.append(&mut self.buffer);
        buffer
    }

    pub(crate) fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
}

impl<W> From<W> for Output<W>
where
    W: Write + Debug
{
    fn from(input: W) -> Self {
        Self::new(input)
    }
}

