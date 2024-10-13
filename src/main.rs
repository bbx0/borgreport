// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn
)]
#![warn(clippy::pedantic)]

use std::{io::IsTerminal, path::PathBuf};

use anyhow::{Context, Result};

use borg::Borg;

use report::{Formattable, Report};
use repository::Repository;
use utils::send_mail;

mod borg;
mod borg_json;
mod cli;
mod format;
mod report;
mod repository;
mod utils;

/// Emit status information to the caller
/// - If a terminal is attached, print a message and return the cursor to the begin of line.
///   The message gets whitespace filled and truncated at 76 chars.
/// - If `NOTIFY_SOCKET` is set, emit the message to systemd
fn emit_progress<T: AsRef<str>>(msg: T) {
    if !cli::args().no_progress {
        // Emit to console, if a terminal is attached
        if std::io::stdin().is_terminal() {
            eprint!("{:<76.76}\r", msg.as_ref());
        }

        // Emit status to systemd, if env NOTIFY_SOCKET is set (and any discard errors)
        let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Status(msg.as_ref())]);
    }
}

/// Collect all *.env files from given directories and return them sorted
fn collect_env_files<'a>(env_dirs: impl IntoIterator<Item = &'a PathBuf>) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for env_dir in env_dirs {
        files.extend(
            std::fs::read_dir(env_dir)
                .context(format!("Cannot open env directory: {env_dir:?}"))?
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry.path().is_file()
                        && entry
                            .path()
                            .extension()
                            .map_or(false, |ext| ext.eq_ignore_ascii_case("env"))
                })
                .map(|entry| entry.path()),
        );
    }
    files.sort_unstable();
    Ok(files)
}

/// Create a report for a single `Repository`
fn create_report(repo: &Repository) -> Report {
    let mut report = Report::new();
    let borg = Borg::from(repo);

    // Process all archive_globs or process `None` when no filter is given
    let mut archive_globs = repo.archive_globs.clone().into_iter().peekable();
    loop {
        let archive_glob = archive_globs.next();

        // Query `borg info` on the repository
        let info_result = borg.info(&archive_glob);

        // If there is a glob, a result but no matching archive then warn about the glob and skip processing.
        if archive_glob.is_some() && info_result.as_ref().is_ok_and(|i| i.archives.is_empty()) {
            report.add_warning(format!(
                "{}: The glob '{}' yields no result!",
                &repo.name,
                archive_glob.unwrap_or_default()
            ));
        } else {
            // Parse the response into the Report
            report.append(Report::from_borg_info_result(&repo.name, &info_result));

            // Perform sanity checks
            if let Ok(info_result) = &info_result {
                report.append(Report::from_sanity_checks(
                    &repo.name,
                    info_result,
                    repo.max_age_hours,
                ));
            }

            // Query `borg check` on the archives
            if repo.run_check {
                match &info_result {
                    Ok(info) if !info.archives.is_empty() => {
                        for archive in &info.archives {
                            report.append(Report::from_borg_check_result(
                                &repo.name,
                                Some(&archive.name),
                                &borg.check(Some(&archive.name)),
                            ));
                        }
                    }
                    // Check the whole repository, when there are no archives found (and no glob was given initially)
                    // -> An empty repository can also be checked.
                    Ok(_) => report.append(Report::from_borg_check_result(
                        &repo.name,
                        None,
                        &borg.check(None),
                    )),
                    Err(_) => {}
                }
            }
        }

        if archive_globs.peek().is_none() {
            break;
        }
    }

    report
}

/// Format the `report` with the given CLI `OutputFormat` and return it as `String`
fn format_report(format: &cli::OutputFormat, report: &Report) -> Result<String> {
    match format {
        cli::OutputFormat::Text => Ok(report.to_string(format::Text)?),
        cli::OutputFormat::Html => Ok(report.to_string(format::Html)?),
    }
}

fn main() -> Result<()> {
    // Collect the command line options
    let args = cli::args();

    // Print extended help and early exit?
    if args.print_help2man {
        cli::print_help2man()?;
        std::process::exit(0);
    }

    // Find all *.env files and parse them into a `Repository` configuration
    let repositories = collect_env_files(&args.env_dirs)?
        .iter()
        .map(Repository::from_env_file)
        .collect::<Result<Vec<Repository>>>()?;

    // Confirm service startup after parsing all files and directories
    sd_notify::notify(false, &[sd_notify::NotifyState::Ready])?;

    let mut report = Report::new();
    if repositories.is_empty() {
        report.add_warning(format!("No *.env files found in {:?}", &args.env_dirs));
    }
    for repo in repositories {
        emit_progress(format!("Process repository: {:?}", &repo.name));
        report.append(create_report(&repo));
        emit_progress("Done."); // This needs to be a short message to get fully overwritten by the next console message.
    }

    // Write report to stdout if not written somewhere else
    let mut output_processed = false;
    let mut force_stdout = false;

    // Write to file ?
    if let Some(file) = &args.output_file {
        if file.to_string_lossy().eq("-") {
            force_stdout = true;
        } else {
            std::fs::write(file, format_report(&args.output_format, &report)?)?;
            output_processed = true;
        }
    }

    // Send report per mail ?
    if let Some(mail_to) = &args.mail_to {
        let mut suffix = vec![];
        if report.has_errors() {
            suffix.push(format!("Errors:{}", report.count_errors()));
        };
        if report.has_warnings() {
            suffix.push(format!("Warnings:{}", report.count_warnings()));
        };
        send_mail(
            mail_to,
            args.mail_from.as_ref(),
            &format!(
                "Backup report ({}) {}",
                jiff::Zoned::now().date(),
                suffix.join(" ")
            ),
            report.to_string(format::Text)?,
            report.to_string(format::Html)?,
        )?;
        output_processed = true;
    }

    // Print to stdout
    if !output_processed || force_stdout {
        print!("{}", format_report(&args.output_format, &report)?);
    };

    // Announce service shutdown, if we are a systemd service
    sd_notify::notify(false, &[sd_notify::NotifyState::Stopping])?;

    Ok(())
}
