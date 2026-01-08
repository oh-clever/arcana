// The core library for Arcana template parsing and compilation.
// Copyright (C) 2026  OC (oc@oh-clever.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod context;
mod error;
mod input;
mod macros;
mod output;
mod parser;

pub use {
    context::Context,
    error::{ InternalResult, InternalError, },
};

use {
    crate::{
        input::TryIntoInput,
        parser::TemplateParser,
    },
    std::{
        fmt::Debug,
        io::{ Read, stdout, Write, },
        path::Path,
    },
};

/// The Arcana compiler.
pub struct Arcana;
impl Arcana {
    /// Compile the input template to a given output with a specific starting
    /// context.
    ///
    /// # Arguments
    ///
    /// * `input` - The [readable](Read) template.
    /// * `output` - The [writable](Write) output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::{ Context, Arcana, };
    /// 
    /// let mut ctx = Context::default();
    /// ctx.add_variable("x", "./", "5");
    /// let input = "{% set i %}{% add x %}0{% /add %}{% /set %}{{ i }}";
    /// let mut output = Vec::<u8>::new();
    /// Arcana::compile_with_ctx(input, &mut output, ctx).unwrap();
    /// let output_str = String::from_utf8(output).unwrap();
    ///assert_eq!("5", output_str);
    /// ```
    ///
    pub fn compile_with_ctx<R, I, W>(input: I, output: W, ctx: Context) -> InternalResult<()>
    where
        R: Read + Debug,
        I: TryIntoInput<R>,
        W: Write + Debug,
    {
        let input = input.try_into_input()?;
        let mut parser = TemplateParser::new(ctx, input, output)?;

        parser.parse()?;

        Ok(())
    }

    /// Compile the input template to a given output.
    ///
    /// # Arguments
    ///
    /// * `input` - The [readable](Read) template.
    /// * `output` - The [writable](Write) output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::Arcana;
    /// 
    /// let input = "{% set i %}0{% /set %}{{ i }}";
    /// let mut output = Vec::<u8>::new();
    /// Arcana::compile(input, &mut output).unwrap();
    /// let output_str = String::from_utf8(output).unwrap();
    ///assert_eq!("0", output_str);
    /// ```
    ///
    pub fn compile<R, I, W>(input: I, output: W) -> InternalResult<()>
    where
        R: Read + Debug,
        I: TryIntoInput<R>,
        W: Write + Debug,
    {
        Self::compile_with_ctx(input, output, Context::default())
    }

    /// Compile a template file to a given output with a specific context.
    ///
    /// # Arguments
    ///
    /// * `path` - The [path](Path) to the file.
    /// * `output` - The [writable](Write) output.
    /// * `ctx` - The [context](Context).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::{ Context, Arcana, };
    ///
    /// let ctx = Context::default();
    /// let path = "./resources/parse_file_1/page.arct";
    /// let mut output = Vec::<u8>::new();
    /// Arcana::compile_file_with_ctx(path, &mut output, ctx).unwrap();
    /// let output_str = String::from_utf8(output).unwrap();
    /// assert_eq!("The number: 4", output_str);
    /// ```
    ///
    pub fn compile_file_with_ctx<P, W>(path: P, output: W, ctx: Context) -> InternalResult<()>
    where
        P: AsRef<Path>,
        W: Write + Debug,
    {
        Self::compile_with_ctx(path.as_ref(), output, ctx)
    }

    /// Compile a template file to a given output.
    ///
    /// # Arguments
    ///
    /// * `path` - The [path](Path) to the file.
    /// * `output` - The [writable](Write) output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::Arcana;
    ///
    /// let path = "./resources/parse_file_1/page.arct";
    /// let mut output = Vec::<u8>::new();
    /// Arcana::compile_file(path, &mut output).unwrap();
    /// let output_str = String::from_utf8(output).unwrap();
    /// assert_eq!("The number: 4", output_str);
    /// ```
    ///
    pub fn compile_file<P, W>(path: P, output: W) -> InternalResult<()>
    where
        P: AsRef<Path>,
        W: Write + Debug,
    {
        Self::compile(path.as_ref(), output)
    }

    /// Compile an input template to stdout with a specific context.
    ///
    /// # Arguments
    ///
    /// * `input` - The [readable](Read) template.
    /// * `ctx` - The [context](Context).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::{ Context, Arcana, };
    /// 
    /// let ctx = Context::default();
    /// let input = "{% set i %}0{% /set %}{{ i }}";
    /// Arcana::compile_to_stdout_with_ctx(input, ctx).unwrap();
    /// ```
    ///
    pub fn compile_to_stdout_with_ctx<R, I>(input: I, ctx: Context) -> InternalResult<()>
    where
        R: Read + Debug,
        I: TryIntoInput<R>,
    {
        Self::compile_with_ctx(input, stdout(), ctx)
    }

    /// Compile an input template to stdout.
    ///
    /// # Arguments
    ///
    /// * `input` - The [readable](Read) template.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::Arcana;
    /// 
    /// let input = "{% set i %}0{% /set %}{{ i }}";
    /// Arcana::compile_to_stdout(input).unwrap();
    /// ```
    ///
    pub fn compile_to_stdout<R, I>(input: I) -> InternalResult<()>
    where
        R: Read + Debug,
        I: TryIntoInput<R>,
    {
        Self::compile(input, stdout())
    }

    /// Compile a template file to [stdout](std::io::Stdout) with a specific
    /// context.
    ///
    /// # Arguments
    ///
    /// * `path` - The [path](Path) to the template.
    /// * `context` - The [context](Context).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::{ Context, Arcana, };
    ///
    /// Arcana::compile_file_to_stdout_with_ctx(
    ///         "./resources/parse_file_1/page.arct",
    ///         Context::default(),
    ///         )
    ///     .unwrap();
    /// ```
    ///
    pub fn compile_file_to_stdout_with_ctx<P>(path: P, ctx: Context) -> InternalResult<()>
    where
        P: AsRef<Path>,
    {
        Self::compile_file_with_ctx(path, stdout(), ctx)
    }

    /// Compile a template file to [stdout](std::io::Stdout).
    ///
    /// # Arguments
    ///
    /// * `path` - The [path](Path) to the template.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use arcana_core::Arcana;
    ///
    /// Arcana::compile_file_to_stdout("./resources/parse_file_1/page.arct")
    ///     .unwrap();
    /// ```
    ///
    pub fn compile_file_to_stdout<P>(path: P) -> InternalResult<()>
    where
        P: AsRef<Path>,
    {
        Self::compile_file(path, stdout())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_file_1() {
        let mut output = Vec::<u8>::new();
        crate::Arcana::compile_file("./resources/parse_file_1/page.arct", &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!("The number: 4", output);
    }
}
