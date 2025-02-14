// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

mod html;
mod metrics;
mod text;

use crate::report::Component;

pub use html::Html;
pub use metrics::Metrics;
pub use text::Text;

/// Format a `T` with the `Formatter`
pub trait Formatter<T>
where
    T: Component,
{
    /// Format data `T` and write result into a String buffer `buf`
    fn format<W>(buf: &mut W, data: &T) -> std::fmt::Result
    where
        W: std::fmt::Write;
}

/// Provide methods to format Report Components
pub trait Formattable: Sized + Component {
    /// Format with `F` and write result into a String buffer `buf`
    fn format<F, W>(&self, buf: &mut W, _f: F) -> std::fmt::Result
    where
        F: Formatter<Self>,
        W: std::fmt::Write,
    {
        F::format(buf, self)
    }

    /// Format with `F` and return result as `String`
    fn to_string<F>(&self, f: F) -> std::result::Result<String, std::fmt::Error>
    where
        F: Formatter<Self>,
    {
        let mut buf = String::new();
        self.format(&mut buf, f)?;
        Ok(buf)
    }
}

/// All `ReportComponent`s can be formatted
impl<T> Formattable for T where T: Sized + Component {}
