// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

mod html;
mod metrics;
mod text;

pub type TextFmt<'a, T> = FormatAdapter<'a, T, text::Text>;
pub type HtmlFmt<'a, T> = FormatAdapter<'a, T, html::Html>;
pub type MetricsFmt<'a, T> = FormatAdapter<'a, T, metrics::Metrics>;

/// Format a `T` with the `Formatter`
pub trait Formatter<T> {
    /// Write formatted data and into a `std::fmt::Write` buffer
    fn format<W>(buf: &mut W, data: &T) -> std::fmt::Result
    where
        W: std::fmt::Write;
}

/// An adapter for `std::fmt::Display` and `std::io::Write` to print data with the `Formatter`
///
/// To be used with one of the alias shorthands:
///
/// * `TextFmt`
/// * `HtmlFmt`
/// * `MetricsFmt`
///
/// Example:
///
/// ```rust
/// print!("{}", TextFmt::new(&Report::new()));
/// HtmlFmt::new(&Report::new()).write_file(path)?;
/// ```
#[repr(transparent)]
pub struct FormatAdapter<'a, T, F> {
    data: &'a T,
    _phantom: std::marker::PhantomData<F>,
}
impl<'a, T, F> FormatAdapter<'a, T, F> {
    pub const fn new(data: &'a T) -> Self {
        Self {
            data,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Write formatted data into a `std::io::Write` buffer while preserving `io::Error` messages
    pub fn write<W: std::io::Write>(&self, mut w: W) -> std::io::Result<()>
    where
        F: Formatter<T>,
    {
        let mut fmt_buf = WriteFmtAdapter::from(&mut w);
        match F::format(&mut fmt_buf, self.data) {
            Ok(()) => Ok(()),
            Err(fmt_error) => Err(fmt_buf
                .io_error
                .unwrap_or_else(|| std::io::Error::other(fmt_error))),
        }
    }

    /// Write the formatted data into a file. A `-` as path will redirect to `stdout`.
    pub fn write_file(&self, path: &std::path::PathBuf) -> std::io::Result<()>
    where
        F: Formatter<T>,
    {
        if path.to_string_lossy().eq("-") {
            print!("{self}");
        } else {
            self.write(std::fs::File::create(path)?)?;
        }

        Ok(())
    }
}
impl<T, F> std::fmt::Display for FormatAdapter<'_, T, F>
where
    F: Formatter<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        F::format(f, self.data)
    }
}

/// A shim between `std::fmt::Write` and `std::io::Write`.
/// The last `io::Error` is stored in `io_error` as `fmt::Write` returns `fmt::Error` with no message.
struct WriteFmtAdapter<'a, W>
where
    W: std::io::Write,
{
    io_writer: &'a mut W,
    io_error: Option<std::io::Error>,
}
impl<'a, W: std::io::Write> From<&'a mut W> for WriteFmtAdapter<'a, W> {
    fn from(w: &'a mut W) -> Self {
        Self {
            io_writer: w,
            io_error: None,
        }
    }
}
impl<W: std::io::Write> std::fmt::Write for WriteFmtAdapter<'_, W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self.io_writer.write_all(s.as_bytes()) {
            Ok(()) => {
                self.io_error = None;
                Ok(())
            }
            Err(e) => {
                self.io_error = Some(e);
                Err(std::fmt::Error)
            }
        }
    }
}
