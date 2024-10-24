// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Formattable, Formatter};
use crate::report::{BulletPoint, ChecksEntry, Report, Section, SummaryEntry};
use human_repr::{HumanCount, HumanDuration};

/// Html `Formatter` (text/html)
pub struct Html;
impl Formatter<Report> for Html {
    fn format<W>(buf: &mut W, data: &Report) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let now = jiff::Zoned::now();

        let title = format!(
            "Backup report ({})",
            jiff::fmt::strtime::format("%F", &now).unwrap_or_default()
        );

        // Header and Title
        write!(
            buf,
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset=utf-8>
        <meta name=generator content="{} {}">
        <meta name=license content="{}">
        <meta name=viewport content="width=device-width, initial-scale=1, minimum-scale=1">
        <title>{title}</title>
        <style>
            body {{
                font-family: sans-serif;
            }}
            li {{
                font-family: monospace, sans-serif;
            }}
            table {{ 
                border-collapse: collapse;
                table-layout: fixed;
            }}
            thead {{
                text-align: left;
            }}
            th, td {{
                padding: 5px;
                white-space: nowrap;
            }}
            td {{ 
                border: 1px solid black;
                font-family: monospace, sans-serif;
            }}
        </style>
    </head>
    <body>
        <h1>{title}</h1>"#,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_LICENSE"),
        )?;

        if data.has_errors() {
            write!(
                buf,
                r#"
        <h2>Errors</h2>"#
            )?;
            data.errors.format(buf, Self)?;
        }

        if data.has_warnings() {
            write!(
                buf,
                r#"
        <h2>Warnings</h2>"#
            )?;
            data.warnings.format(buf, Self)?;
        }

        if !data.summary.is_empty() {
            write!(
                buf,
                r#"
        <h2>Summary</h2>"#
            )?;
            data.summary.format(buf, Self)?;
        }

        if !data.checks.is_empty() {
            write!(
                buf,
                r#"
        <h2><code>borg check</code> result</h2>"#
            )?;
            data.checks.format(buf, Self)?;
        }

        // Footer
        write!(
            buf,
            r#"
        <footer>
            <p>
                Generated on {} with <a href="{}" target="_blank">{}</a> {}
            </p>
        </footer>
    </body>
</html>
"#,
            jiff::fmt::rfc2822::to_string(&now).unwrap_or_default(),
            env!("CARGO_PKG_REPOSITORY"),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        )
    }
}

impl Formatter<Section<BulletPoint>> for Html {
    fn format<W>(buf: &mut W, data: &Section<BulletPoint>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        // Print all lines of the section entry under one bullet point
        write!(
            buf,
            r#"
        <ul>"#
        )?;
        for entry in data.dedup_inner() {
            let mut lines = entry.trim().lines();
            if let Some(line) = lines.next() {
                write!(
                    buf,
                    r#"
            <li>{line}"#
                )?;
            }
            for line in lines {
                write!(buf, r#"<br>{line}"#)?;
            }
            write!(buf, r#"</li>"#)?;
        }
        write!(
            buf,
            r#"
        </ul>"#
        )?;
        Ok(())
    }
}

impl Formatter<Section<SummaryEntry>> for Html {
    fn format<W>(buf: &mut W, data: &Section<SummaryEntry>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(
            buf,
            r#"
        <table>
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Hostname</th>
                    <th>Last archive</th>
                    <th>Start</th>
                    <th>Duration</th>
                    <th>Source</th>
                    <th>Δ Archive</th>
                    <th>∑ Repository</th>
                </tr>
            </thead>
            <tbody>"#
        )?;

        for e in data.inner() {
            write!(
                buf,
                r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td style="text-align:right">{}</td>
                    <td style="text-align:right">{}</td>
                    <td style="text-align:right">{}</td>
                    <td style="text-align:right">{}</td>
                </tr>"#,
                e.repository,
                e.hostname,
                e.archive,
                jiff::fmt::strtime::format("%F", e.start).unwrap_or_else(|_| String::default()),
                e.duration.human_duration(),
                e.original_size.human_count_bytes(),
                e.deduplicated_size.human_count_bytes(),
                e.unique_csize.human_count_bytes()
            )?;
        }

        write!(
            buf,
            r#"
            <tbody>
        </table>"#
        )?;

        Ok(())
    }
}

impl Formatter<Section<ChecksEntry>> for Html {
    fn format<W>(buf: &mut W, data: &Section<ChecksEntry>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(
            buf,
            r#"
        <table>
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Archive</th>
                    <th>Duration</th>
                    <th>Okay</th>
                </tr>
            </thead>
            <tbody>"#
        )?;

        for e in data.inner() {
            write!(
                buf,
                r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td style="text-align:right">{}</td>
                    <td style="text-align:right">{}</td>
                </tr>"#,
                e.repository,
                e.archive_name.clone().unwrap_or_default(),
                e.duration.human_duration(),
                if e.status.success() { "yes" } else { "no" }
            )?;
        }

        write!(
            buf,
            r#"
            <tbody>
        </table>"#
        )?;

        Ok(())
    }
}
