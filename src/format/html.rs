// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Formattable, Formatter};
use crate::report::{BulletPoint, ChecksEntry, CompactsEntry, Report, Section, SummaryEntry};
use human_repr::{HumanCount, HumanDuration};

/// Html `Formatter` (text/html)
pub struct Html;
impl Formatter<Report> for Html {
    #![allow(clippy::too_many_lines)]
    fn format<W>(buf: &mut W, data: &Report) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        let now = jiff::Zoned::now();

        let title = format!("Backup report ({})", now.date());

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
            code {{
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
                r"
        <h2>Errors</h2>"
            )?;
            data.errors.format(buf, Self)?;
        }

        if data.has_warnings() {
            write!(
                buf,
                r"
        <h2>Warnings</h2>"
            )?;
            data.warnings.format(buf, Self)?;
        }

        if !data.summary.is_empty() {
            write!(
                buf,
                r"
        <h2>Summary</h2>"
            )?;
            data.summary.format(buf, Self)?;
        }

        if !data.checks.is_empty() {
            write!(
                buf,
                r"
        <h2><code>borg check</code> result</h2>"
            )?;
            data.checks.format(buf, Self)?;
        }

        if !data.compacts.is_empty() {
            write!(
                buf,
                r"
        <h2><code>borg compact</code> result</h2>"
            )?;
            data.compacts.format(buf, Self)?;
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
            now.strftime("%a, %d %b %Y %T %z"),
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
            r"
        <ul>"
        )?;
        for entry in data.dedup_inner() {
            let mut lines = entry.trim().lines();
            if let Some(line) = lines.next() {
                write!(
                    buf,
                    r"
            <li>{line}"
                )?;
            }
            for line in lines {
                write!(buf, r"<br>{line}")?;
            }
            write!(buf, r"</li>")?;
        }
        write!(
            buf,
            r"
        </ul>"
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
            r"
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
            <tbody>"
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
                if e.start.timestamp().is_zero() {
                    jiff::civil::Date::ZERO
                } else {
                    e.start.with_time_zone(jiff::tz::TimeZone::system()).date()
                },
                e.duration.as_secs_f64().human_duration(),
                e.original_size.human_count_bytes(),
                e.deduplicated_size.human_count_bytes(),
                e.unique_csize.human_count_bytes()
            )?;
        }

        write!(
            buf,
            r"
            <tbody>
        </table>"
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
            r"
        <table>
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Archive</th>
                    <th>Duration</th>
                    <th>Okay</th>
                </tr>
            </thead>
            <tbody>"
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
                e.duration.as_secs_f64().human_duration(),
                if e.status.success() { "yes" } else { "no" }
            )?;
        }

        write!(
            buf,
            r"
            <tbody>
        </table>"
        )?;

        Ok(())
    }
}

impl Formatter<Section<CompactsEntry>> for Html {
    fn format<W>(buf: &mut W, data: &Section<CompactsEntry>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        if data.iter().any(|r| r.freed_bytes.is_none()) {
            write!(
                buf,
                r"
        <p>Repositories with errors or warnings are not compacted.</p>"
            )?;
        }

        write!(
            buf,
            r"
        <table>
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Duration</th>
                    <th>Freed space</th>
                </tr>
            </thead>
            <tbody>"
        )?;

        for r in data.inner() {
            if r.freed_bytes.is_none() {
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{}</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                </tr>"#,
                    r.repository
                )?;
            } else {
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{}</td>
                    <td style="text-align:right">{}</td>
                    <td style="text-align:right">{}</td>
                </tr>"#,
                    r.repository,
                    r.duration.as_secs_f64().human_duration(),
                    r.freed_bytes.unwrap_or_default().human_count_bytes()
                )?;
            }
        }

        write!(
            buf,
            r"
            <tbody>
        </table>"
        )
    }
}
