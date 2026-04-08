// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Formatter;
use crate::report::{Listed, Report, Tabular, TabularCellAlignment};

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
        <meta charset="utf-8">
        <meta name="generator" content="{} {}">
        <meta name="license" content="{}">
        <meta name="viewport" content="width=device-width, initial-scale=1, minimum-scale=1">
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
            format_listed(buf, &data.errors)?;
        }

        if data.has_warnings() {
            write!(
                buf,
                r"
        <h2>Warnings</h2>"
            )?;
            format_listed(buf, &data.warnings)?;
        }

        if !data.summary.is_empty() {
            write!(
                buf,
                r"
        <h2>Summary</h2>"
            )?;
            format_tabular(buf, &data.summary)?;
        }

        if !data.checks.is_empty() {
            write!(
                buf,
                r"
        <h2><code>borg check</code> result</h2>"
            )?;
            format_tabular(buf, &data.checks)?;
        }

        if !data.compacts.is_empty() {
            write!(
                buf,
                r"
        <h2><code>borg compact</code> result</h2>"
            )?;
            format_tabular(buf, &data.compacts)?;
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
        )?;

        Ok(())
    }
}

fn format_listed<T: Listed, W: std::fmt::Write>(buf: &mut W, data: &T) -> std::fmt::Result {
    // Print all lines of the section entry under one bullet point
    write!(
        buf,
        r"
        <ul>"
    )?;
    for entry in data.list_iter() {
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

fn format_tabular<T: Tabular, W: std::fmt::Write>(buf: &mut W, data: &T) -> std::fmt::Result {
    let alignment: Vec<&str> = T::table_alignment()
        .iter()
        .map(|a| match a {
            TabularCellAlignment::Left => "left",
            TabularCellAlignment::Right => "right",
        })
        .collect();

    for note in data.table_preface() {
        write!(
            buf,
            r"
        <p>{note}</p>"
        )?;
    }
    write!(
        buf,
        r"
        <table>
            <thead>
                <tr>"
    )?;
    for head in T::table_header() {
        write!(
            buf,
            r"
                    <th>{head}</th>"
        )?;
    }
    write!(
        buf,
        r"
                </tr>
            </thead>
            <tbody>"
    )?;
    for row in data.table_row_iter() {
        write!(
            buf,
            r"
                <tr>"
        )?;
        for (i, col) in row.iter().enumerate() {
            write!(
                buf,
                r#"
                    <td style="text-align:{}">{col}</td>"#,
                alignment[i]
            )?;
        }
        write!(
            buf,
            r"
                </tr>",
        )?;
    }
    write!(
        buf,
        r"
            </tbody>
        </table>"
    )?;

    Ok(())
}
