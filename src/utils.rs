// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{io::Write, str::FromStr};

use anyhow::{Context, Result, bail};
use email_address::EmailAddress;

/// carriage return (CR) character
const CR: char = '\r';
/// line feed (LF) character
const LF: char = '\n';
/// carriage return (CR) + line feed (LF) pair
const CRLF: &str = "\r\n";
/// sendmail executable
const SENDMAIL: &str = "sendmail";

/// A simple `sendmail` wrapper expecting the body in plain text and html format
pub fn send_mail(
    to: &EmailAddress,
    from: Option<&EmailAddress>,
    subject: &str,
    plain: &str,
    html: &str,
) -> Result<()> {
    /// MIME multipart boundary (must be unique)
    const BOUNDARY: &str = "cmVzcGVjdCBvdGhlciBwZW9wbGUncyBib3VuZGFyaWVz";
    if plain.contains(BOUNDARY) || html.contains(BOUNDARY) {
        bail!(
            "Email cannot be sent! The report must not contain string {BOUNDARY} used as multipart boundary."
        )
    }

    // Current timestamp in RFC 2822 format (constructed to not panic on error)
    let now = jiff::fmt::rfc2822::to_string(&jiff::Zoned::try_from(std::time::SystemTime::now())?)?;

    // The message must contain a from address
    // Prepare a default {username}@{hostname} sender address with fallback to CARGO_PKG_NAME@localhost
    let message_from = from.cloned().unwrap_or_else(|| {
        if let (Ok(username), Ok(hostname)) =
            (whoami::fallible::username(), whoami::fallible::hostname())
            && let Ok(from) = EmailAddress::from_str(format!("{username}@{hostname}").as_str())
        {
            from
        } else {
            EmailAddress::new_unchecked(format!("{}@localhost", env!("CARGO_PKG_NAME")))
        }
    });

    // Lines must end with CRLF to comply with RFC 2822
    let message = format!(
        "\
From: {message_from}{CR}
To: {to}{CR}
Subject: {subject}{CR}
MIME-Version: 1.0{CR}
Date: {now}{CR}
Content-Type: multipart/alternative;{CR}
 boundary={BOUNDARY}{CR}
{CR}
--{BOUNDARY}{CR}
Content-Type: text/plain; charset=utf-8{CR}
Content-Transfer-Encoding: quoted-printable{CR}
{CR}
{}{CR}
{CR}
--{BOUNDARY}{CR}
Content-Type: text/html; charset=utf-8{CR}
Content-Transfer-Encoding: quoted-printable{CR}
{CR}
{}{CR}
{CR}
--{BOUNDARY}--{CR}
",
        quoted_printable::encode_to_str(plain.replace(LF, CRLF)),
        quoted_printable::encode_to_str(html.replace(LF, CRLF))
    );

    // call sendmail in form of: echo message | sendmail [-f <from@sender>] -- <to@receiver>
    let (stderr_rx, stderr_tx) = std::io::pipe()?;
    if !std::process::Command::new(SENDMAIL)
        // the from address in the envelope is optional
        .args(from.map_or_else(std::vec::Vec::new, |from| vec!["-f", from.as_str()]))
        // the to address is mandatory
        .args(["--", to.as_str()])
        // pipe the message to stdin
        .stdin({
            let (rx, mut tx) = std::io::pipe()?;
            write!(tx, "{message}")?;
            rx
        })
        .stdout(stderr_tx.try_clone()?)
        .stderr(stderr_tx)
        .status()
        .context("Failed to execute sendmail")?
        .success()
    {
        bail!(std::io::read_to_string(stderr_rx)?);
    }

    Ok(())
}

/// Find the first typed byte value in a string (e.g. "Some text 3.4 kB")
pub fn first_typed_bytes(input: &str) -> Option<u64> {
    // Find the first float value and try to parse it with the next word as an SI unit
    let mut words = input.split_whitespace().peekable();
    while let (Some(value), Some(unit)) = (words.next(), words.peek()) {
        if let Ok(Ok(value)) = value
            .parse::<f64>()
            .map(|value| format!("{value}{unit}").parse::<typed_bytesize::ByteSizeSi>())
        {
            return Some(value.into());
        }
    }
    None
}
