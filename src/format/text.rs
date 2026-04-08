// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Formatter;
use crate::report::{Listed, Report, Tabular, TabularCellAlignment};
use comfy_table::{CellAlignment, ContentArrangement, Table, presets::ASCII_MARKDOWN};

/// Text `Formatter` (text/plain)
pub struct Text;
impl Formatter<Report> for Text {
    fn format<W>(buf: &mut W, data: &Report) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let now = jiff::Zoned::now();

        // Title
        writeln!(buf, "==== Backup report ({}) ====\n", now.date())?;

        if data.has_errors() {
            writeln!(buf, "=== Errors ===\n")?;
            format_listed(buf, &data.errors)?;
            writeln!(buf)?;
        }
        if data.has_warnings() {
            writeln!(buf, "=== Warnings ===\n")?;
            format_listed(buf, &data.warnings)?;
            writeln!(buf)?;
        }
        if !data.summary.is_empty() {
            writeln!(buf, "=== Summary ===\n")?;
            format_tabular(buf, &data.summary)?;
            writeln!(buf)?;
        }
        if !data.checks.is_empty() {
            writeln!(buf, "=== `borg check` result ===\n")?;
            format_tabular(buf, &data.checks)?;
            writeln!(buf)?;
        }

        if !data.compacts.is_empty() {
            writeln!(buf, "=== `borg compact` result ===\n")?;
            format_tabular(buf, &data.compacts)?;
            writeln!(buf)?;
        }

        // Footer
        writeln!(
            buf,
            "Generated {} ({} {})",
            now.strftime("%a, %d %b %Y %T %z"),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
    }
}

fn format_listed<T: Listed, W: std::fmt::Write>(buf: &mut W, data: &T) -> std::fmt::Result {
    // Print all lines of the section entry and add a bullet point to its first line
    for item in data.list_iter() {
        let mut lines = item.trim().lines();
        if let Some(line) = lines.next() {
            writeln!(buf, " * {line}")?;

            for line in lines {
                writeln!(buf, "   {line}")?;
            }
        }
    }
    Ok(())
}

fn format_tabular<T: Tabular, W: std::fmt::Write>(buf: &mut W, data: &T) -> std::fmt::Result {
    for note in data.table_preface() {
        writeln!(buf, "{note}\n")?;
    }

    let mut table = Table::new();
    table
        .load_preset(ASCII_MARKDOWN)
        .set_content_arrangement(ContentArrangement::Disabled)
        .set_header(T::table_header());

    for row in data.table_row_iter() {
        table.add_row(row);
    }

    for (i, align) in T::table_alignment().iter().enumerate() {
        if let Some(column) = table.column_mut(i) {
            column.set_cell_alignment(match align {
                TabularCellAlignment::Left => CellAlignment::Left,
                TabularCellAlignment::Right => CellAlignment::Right,
            });
        }
    }

    writeln!(buf, "{table}")
}
