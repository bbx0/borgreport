// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use assert_cmd::Command;

pub mod init;

/// Borg binary to use
pub const BORG_BIN: &str = "borg";

/// Command under test
pub fn cargo_bin() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
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
pub fn borg_create<'a, I>(repo: &str, archive: &str, options: I)
where
    I: IntoIterator<Item = &'a str>,
{
    Command::new(BORG_BIN)
        .env_clear()
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .arg("create")
        .args(options)
        .args(["--stdin-name", "file.txt", &format!("::{archive}"), "-"])
        .write_stdin(test_data_32_bytes_n(125)) // 4000 bytes
        .assert()
        .success();
}

/// Create an empty archive in a repository
pub fn borg_create_empty<'a, I>(repo: &str, archive: &str, options: I)
where
    I: IntoIterator<Item = &'a str>,
{
    Command::new(BORG_BIN)
        .env_clear()
        .envs(DEFAULT_ENV)
        .env("BORG_REPO", repo)
        .arg("create")
        .args(options)
        .args([&format!("::{archive}"), "-"])
        .assert()
        .success();
}

/// Return `32 * n` bytes of dummy data.
fn test_data_32_bytes_n(n: usize) -> Vec<u8> {
    std::iter::repeat_n(*b"eW91Zm91bmR0aGVlYXN0ZXJlZ2chCg==", n)
        .flatten()
        .collect()
}
