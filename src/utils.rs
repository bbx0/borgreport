// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use lettre::{
    address::Envelope,
    message::{header::ContentType, SinglePart},
    Address, Message, SendmailTransport, Transport,
};

/// A simple `sendmail` wrapper
pub fn send_mail(to: &Address, from: Option<&Address>, subject: &str, body: String) -> Result<()> {
    // Provide a default sender address if `None` is given
    let from_checked = match from {
        Some(from) => from.clone(),
        None => Address::new(
            whoami::fallible::username().unwrap_or(env!("CARGO_PKG_NAME").to_string()),
            whoami::fallible::hostname().unwrap_or("localhost".to_string()),
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
        .singlepart(
            SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(body),
        )?;
    SendmailTransport::new().send(&message)?;
    Ok(())
}
