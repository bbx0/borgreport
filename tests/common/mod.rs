// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use assert_cmd::Command;

pub mod init;

//pub type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>;

/// Borg binary to use
pub const BORG_BIN: &str = "borg";

/// Default env for borgreport and borg.
/// Attention: `BORG_BASE_DIR`can require a `sealed_test`!
pub const DEFAULT_ENV: [(&str, &str); 3] = [
    ("BORG_BASE_DIR", "borg"),
    ("BORG_PASSPHRASE", "passphrase"),
    ("BORGREPORT_CHECK", "true"),
];

/// Shorthand for the Directory|Path Separator
pub const DS: &str = std::path::MAIN_SEPARATOR_STR;

/// Init a new repository
pub fn borg_init(repo: &str) {
    Command::new(BORG_BIN)
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .args(["init", "--encryption=repokey"])
        .assert()
        .success();
}

/// Create an archive in a repository
pub fn borg_create<'a, I>(repo: &str, archive: &str, options: I)
where
    I: IntoIterator<Item = &'a str>,
{
    Command::new(BORG_BIN)
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .arg("create")
        .args(options)
        .args([&format!("::{archive}"), &format!("{repo}{DS}config")])
        .assert()
        .success();
}

/// Command under test
pub fn cargo_bin() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}
