// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Formattable, Formatter};
use crate::report::{BulletPoint, ChecksEntry, Report, Section, SummaryEntry};
use comfy_table::{CellAlignment, ContentArrangement, Table, presets::ASCII_MARKDOWN};
use human_repr::{HumanCount, HumanDuration};

/// Text `Formatter` (text/plain)
pub struct Text;
impl Formatter<Report> for Text {
    fn format<W>(buf: &mut W, data: &Report) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let now = jiff::Zoned::now();

        // Title
        writeln!(
            buf,
            "==== Backup report ({}) ====\n",
            jiff::fmt::strtime::format("%F", &now).unwrap_or_default(),
        )?;

        if data.has_errors() {
            writeln!(buf, "=== Errors ===\n\n{}", data.errors.to_string(Self)?)?;
        }
        if data.has_warnings() {
            writeln!(
                buf,
                "=== Warnings ===\n\n{}",
                data.warnings.to_string(Self)?
            )?;
        }
        if !data.summary.is_empty() {
            writeln!(buf, "=== Summary ===\n\n{}", data.summary.to_string(Self)?)?;
        }
        if !data.checks.is_empty() {
            writeln!(
                buf,
                "=== `borg check` result ===\n\n{}",
                data.checks.to_string(Self)?,
            )?;
        }

        // Footer
        writeln!(
            buf,
            "Generated {} ({} {})",
            jiff::fmt::rfc2822::to_string(&now).unwrap_or_default(),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
    }
}

impl Formatter<Section<BulletPoint>> for Text {
    fn format<W>(buf: &mut W, data: &Section<BulletPoint>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        // Print all lines of the section entry and add a bullet point to its first line
        for entry in data.dedup_inner() {
            let mut lines = entry.trim().lines();
            if let Some(line) = lines.next() {
                writeln!(buf, " * {line}")?;
            }
            for line in lines {
                writeln!(buf, "   {line}")?;
            }
        }
        Ok(())
    }
}

impl Formatter<Section<SummaryEntry>> for Text {
    fn format<W>(buf: &mut W, data: &Section<SummaryEntry>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let mut table = Table::new();
        table
            .load_preset(ASCII_MARKDOWN)
            .set_content_arrangement(ContentArrangement::Disabled)
            .set_header(vec![
                "Repository",
                "Hostname",
                "Last archive",
                "Start",
                "Duration",
                "Source",
                "Δ Archive",
                "∑ Repository",
            ]);
        for e in data.inner() {
            table.add_row(vec![
                format!("{}", e.repository),
                format!("{}", e.hostname),
                format!("{}", e.archive),
                jiff::fmt::strtime::format("%F", e.start).unwrap_or_else(|_| String::default()),
                format!("{}", e.duration.as_secs_f64().human_duration()),
                format!("{}", e.original_size.human_count_bytes()),
                format!("{}", e.deduplicated_size.human_count_bytes()),
                format!("{}", e.unique_csize.human_count_bytes()),
            ]);
        }
        //the columns 4,5,6,7 are aligned right
        for i in 4..=7 {
            if let Some(c) = table.column_mut(i) {
                c.set_cell_alignment(CellAlignment::Right);
            }
        }
        writeln!(buf, "{table}")
    }
}

impl Formatter<Section<ChecksEntry>> for Text {
    fn format<W>(buf: &mut W, data: &Section<ChecksEntry>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let mut table = Table::new();
        table
            .load_preset(ASCII_MARKDOWN)
            .set_content_arrangement(ContentArrangement::Disabled)
            .set_header(vec!["Repository", "Archive", "Duration", "Okay"]);
        for e in data.inner() {
            table.add_row(vec![
                format!("{}", e.repository),
                format!("{}", e.archive_name.clone().unwrap_or_default()),
                format!("{}", e.duration.as_secs_f64().human_duration()),
                format!("{}", if e.status.success() { "yes" } else { "no" }),
            ]);
        }
        //columns 2,3 are aligned right
        for i in 2..=3 {
            if let Some(c) = table.column_mut(i) {
                c.set_cell_alignment(CellAlignment::Right);
            }
        }
        writeln!(buf, "{table}")
    }
}
