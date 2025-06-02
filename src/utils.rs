// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use lettre::{
    Address, Message, SendmailTransport, Transport, address::Envelope, message::MultiPart,
};

/// A simple `sendmail` wrapper expecting the body in plain text and html format
pub fn send_mail(
    to: &Address,
    from: Option<&Address>,
    subject: &str,
    plain: String,
    html: String,
) -> Result<()> {
    // Provide a default sender address if `None` is given
    let from_checked = match from {
        Some(from) => from.clone(),
        None => Address::new(
            whoami::fallible::username().unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
            whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string()),
        )
        .context("Cannot parse fallback mail <from> address")?,
    };

    // `sendmail` does not need a <from> address in the envelope but the `MessageBuilder` enforces it.
    // Use a custom envelope to make it actually optional and have sendmail read it from the header otherwise.
    // This allows a pre-configured <from> address in sendmail itself to take effect.
    let envelope = match from {
        Some(_) => Envelope::new(Some(from_checked.clone()), vec![to.clone()])?,
        None => Envelope::new(None, vec![to.clone()])?,
    };

    let message = Message::builder()
        .from(from_checked.into())
        .to(to.clone().into())
        .envelope(envelope)
        .subject(subject)
        .multipart(MultiPart::alternative_plain_html(plain, html))?;

    SendmailTransport::new().send(&message)?;
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
