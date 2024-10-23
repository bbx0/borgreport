// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

mod html;
mod metrics;
mod text;

use crate::report::Component;

pub(crate) use html::Html;
pub(crate) use metrics::Metrics;
pub(crate) use text::Text;

/// Format a `T` with the `Formatter`
pub trait Formatter<T>
where
    T: Component,
{
    /// Format data `T` and write result into a String buffer `buf`
    fn format(buf: &mut String, data: &T) -> std::fmt::Result;
}

/// Provide methods to format Report Components
pub trait Formattable: Sized + Component {
    /// Format with `F` and write result into a String buffer `buf`
    fn format<F>(&self, buf: &mut String, _f: F) -> std::fmt::Result
    where
        F: Formatter<Self>,
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
