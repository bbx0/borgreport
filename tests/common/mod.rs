// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use assert_cmd::{Command, cargo::cargo_bin_cmd};

pub mod init;

/// Borg binary to use
pub const BORG_BIN: &str = "borg";

/// Command under test
pub fn cargo_bin() -> Command {
    cargo_bin_cmd!()
}

/// Default env for borgreport and borg.
/// Attention: Using the same `BORG_BASE_DIR` can require a `sealed_test`!
pub const DEFAULT_ENV: [(&str, &str); 6] = [
    ("LC_ALL", "C.UTF-8"),
    ("TZ", "UTC"),
    ("BORG_BASE_DIR", "borg"),
    ("BORG_PASSPHRASE", "passphrase"),
    ("BORGREPORT_CHECK", "true"),
    ("BORGREPORT_COMPACT", "true"),
];

/// Shorthand for the Directory|Path Separator
pub const DS: &str = std::path::MAIN_SEPARATOR_STR;

/// Init a new repository
pub fn borg_init(repo: &str) {
    Command::new(BORG_BIN)
        .env_clear()
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .args(["init", "--encryption=repokey"])
        .assert()
        .success();
}

/// Create an archive in a repository
pub fn borg_create<'a, I>(repo: &str, archive: &str, source: File, borg_options: I)
where
    I: IntoIterator<Item = &'a str>,
{
    Command::new(BORG_BIN)
        .env_clear()
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .arg("create")
        .args(borg_options)
        .args(["--stdin-name", &source.name(), &format!("::{archive}"), "-"])
        .write_stdin(source.bytes())
        .assert()
        .success();
}

/// A single test file to be used as backup source
pub enum File {
    /// empty:  0 bytes
    Empty,
    /// a:      1344 bytes
    A,
    /// b:      4000 bytes
    B,
}

impl File {
    /// File contents
    fn bytes(&self) -> Vec<u8> {
        match self {
            Self::Empty => Vec::new(),
            Self::A => b"eW91Zm91bmR0aGVlYXN0ZXJlZ2chCg==".repeat(42),
            Self::B => b"IWdnZXJldHNhZWVodGRudW9mdW95Cg==".repeat(125),
        }
    }
    /// File name
    fn name(&self) -> String {
        match self {
            Self::Empty => "empty",
            Self::A => "a",
            Self::B => "b",
        }
        .to_owned()
    }
}
