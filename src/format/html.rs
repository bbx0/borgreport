// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Formattable, Formatter, fmt_glob_or};
use crate::report::{BulletPointSection, CheckSection, CompactSection, InfoSection, Report};
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

impl Formatter<BulletPointSection> for Html {
    fn format<W>(buf: &mut W, data: &BulletPointSection) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        // Print all lines of the section entry under one bullet point
        write!(
            buf,
            r"
        <ul>"
        )?;
        for entry in data.content() {
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

impl Formatter<InfoSection> for Html {
    fn format<W>(buf: &mut W, section: &InfoSection) -> std::fmt::Result
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

        for row in section.content() {
            if let Some(info) = &row.info {
                if let Some(archive) = &info.archive {
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
                        row.repository,
                        archive.hostname,
                        archive.name,
                        if archive.start.timestamp().is_zero() {
                            jiff::civil::Date::ZERO
                        } else {
                            archive
                                .start
                                .with_time_zone(jiff::tz::TimeZone::system())
                                .date()
                        },
                        archive.duration.as_secs_f64().human_duration(),
                        archive.original_size.human_count_bytes(),
                        archive.deduplicated_size.human_count_bytes(),
                        info.repository.unique_csize.human_count_bytes()
                    )?;
                } else {
                    write!(
                        buf,
                        r#"
                <tr>
                    <td>{}</td>
                    <td></td>
                    <td>{}</td>
                    <td></td>
                    <td style="text-align:right"></td>
                    <td style="text-align:right"></td>
                    <td style="text-align:right"></td>
                    <td style="text-align:right">{}</td>
                </tr>"#,
                        row.repository,
                        fmt_glob_or(row.archive_glob.as_deref(), ""),
                        info.repository.unique_csize.human_count_bytes()
                    )?;
                }
            } else {
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{}</td>
                    <td>-</td>
                    <td>{}</td>
                    <td>-</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                </tr>"#,
                    row.repository,
                    fmt_glob_or(row.archive_glob.as_deref(), "-"),
                )?;
            }
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

impl Formatter<CheckSection> for Html {
    fn format<W>(buf: &mut W, data: &CheckSection) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        if data.iter().any(|r| r.check.is_none()) {
            write!(
                buf,
                r"
        <p>Some repositories could not be checked due to previous errors.</p>"
            )?;
        }

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

        for r in data.content() {
            let repository = &r.repository;
            if let Some(check) = &r.check {
                let duration = check.duration.as_secs_f64().human_duration();
                let archive_name = check.archive_name.clone().unwrap_or_default();
                let status = if check.status.success() { "yes" } else { "no" };
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{repository}</td>
                    <td>{archive_name}</td>
                    <td style="text-align:right">{duration}</td>
                    <td style="text-align:right">{status}</td>
                </tr>"#,
                )?;
            } else {
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{repository}</td>
                    <td>{}</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                </tr>"#,
                    fmt_glob_or(r.archive_glob.as_deref(), "-"),
                )?;
            }
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

impl Formatter<CompactSection> for Html {
    fn format<W>(buf: &mut W, data: &CompactSection) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        if data.iter().any(|r| r.compact.is_none()) {
            write!(
                buf,
                r"
        <p>Repositories with errors or warnings are not compacted.</p>"
            )?;
        }

        if data
            .iter()
            .any(|r| r.compact.as_ref().is_some_and(|e| e.freed_bytes.is_none()))
        {
            write!(
                buf,
                r"
        <p>Some remote repositories cannot return the freed bytes. This happens when the SSH_ORIGINAL_COMMAND is not passed to borg serve.</p>"
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

        for r in data.content() {
            let repository = &r.repository;
            if let Some(compact) = &r.compact {
                let duration = compact.duration.as_secs_f64().human_duration();
                let freed_bytes = compact
                    .freed_bytes
                    .map_or_else(String::new, |b| b.human_count_bytes().to_string());
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{repository}</td>
                    <td style="text-align:right">{duration}</td>
                    <td style="text-align:right">{freed_bytes}</td>
                </tr>"#,
                )?;
            } else {
                write!(
                    buf,
                    r#"
                <tr>
                    <td>{repository}</td>
                    <td style="text-align:right">-</td>
                    <td style="text-align:right">-</td>
                </tr>"#,
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
