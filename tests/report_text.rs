// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Test the text format of the report.
//!
//! Rust has no mechanism to run a global setup for integration tests.
//! A dedicated report per test case is required as it is not possible
//! to init all at once and execute individual tests subsequently.
//!

mod common;
use common::*;
use predicates::prelude::*;
use sealed_test::prelude::*;

/// Borg repo used for all tests.
/// This is a "random" name to allow a test to grep for it.
/// Requires a `sealed_test` or every test uses the same repository.
pub const REPO: &str = "repo-4e3ae97";

/// Command under test against one repo provided by env
pub fn cargo_bin(repo: &str) -> assert_cmd::Command {
    let mut c = common::cargo_bin();
    c.env_clear();
    c.envs(DEFAULT_ENV);
    c.env("BORG_REPO", repo);
    c
}

/// An empty directory is not a valid repository
#[sealed_test]
fn no_init() {
    init::empty_directory(REPO);
    cargo_bin(REPO)
        .assert()
        .stdout(predicate::str::contains("Errors"))
        .stdout(predicate::str::contains(format!(
            "{REPO} is not a valid repository.",
        )));
}

/// A repository without backups shall raise a warning.
/// A repository without backups checks OK.
#[sealed_test]
fn no_backups() {
    init::no_archives(REPO);
    cargo_bin(REPO)
        .assert()
        .stdout(predicate::str::contains("Warnings"))
        .stdout(predicate::str::contains(format!("{REPO}: Repository is empty")).count(1))
        .stdout(predicate::str::contains("borg check"))
        .stdout(predicate::str::contains("yes |").count(1));
}

/// A repository with a valid archive checks OK.
#[sealed_test]
fn check_ok() {
    init::one_archive(REPO, "{utcnow}Z");
    cargo_bin(REPO)
        .env("BORGREPORT_CHECK", "true")
        .env("BORGREPORT_COMPACT", "false")
        .assert()
        .stdout(predicate::str::contains(REPO).count(2))
        .stdout(predicate::str::contains("borg check"))
        .stdout(predicate::str::contains("yes |").count(1));
}

/// A repository with a faulty archive checks NOT OK.
#[sealed_test]
fn check_not_ok() {
    init::faulty_archive(REPO, "{utcnow}Z");
    cargo_bin(REPO)
        .assert()
        .stdout(predicate::str::contains("Errors"))
        .stdout(predicate::str::contains(format!(
            "{REPO}: Data integrity error",
        )))
        .stdout(predicate::str::contains("borg check"))
        .stdout(predicate::str::contains("no |").count(1));
}

/// A repository with two valid archives using globs checks OK.
#[sealed_test]
fn two_archives_ok() {
    init::two_archives(REPO, "etc-{utcnow}Z", "srv-{utcnow}Z");
    cargo_bin(REPO)
        .env("BORGREPORT_GLOB_ARCHIVES", "etc-* srv-*")
        .assert()
        .stdout(predicate::str::contains("etc-").count(2))
        .stdout(predicate::str::contains("srv-").count(2))
        .stdout(predicate::str::contains("borg check").count(1))
        .stdout(predicate::str::contains("yes |").count(2));
}

/// A too old archive raises a warning
#[sealed_test]
fn archive_too_old() {
    init::old_archive(REPO, "{utcnow}Z");
    cargo_bin(REPO)
        .assert()
        .stdout(predicate::str::contains("Warnings"))
        .stdout(predicate::str::contains(format!("{REPO}: Last backup is older than")).count(1))
        .stdout(predicate::str::contains("borg check"))
        .stdout(predicate::str::contains("yes |").count(1));
}

/// A repository with two valid archives using globs compacts OK once.
#[sealed_test]
fn compact_ok() {
    init::two_archives(REPO, "etc-{utcnow}Z", "srv-{utcnow}Z");
    cargo_bin(REPO)
        .env("BORGREPORT_GLOB_ARCHIVES", "etc-* srv-*")
        .env("BORGREPORT_CHECK", "false")
        .env("BORGREPORT_COMPACT", "true")
        .assert()
        .stdout(predicate::str::contains("etc-").count(1))
        .stdout(predicate::str::contains("srv-").count(1))
        .stdout(predicate::str::contains(REPO).count(3))
        .stdout(predicate::str::contains("borg compact").count(1));
}

/// A repository with a faulty archive compacts NOT OK when check has failed.
#[sealed_test]
fn compact_not_ok() {
    init::faulty_archive(REPO, "{utcnow}Z");
    cargo_bin(REPO)
        .env("BORGREPORT_CHECK", "true")
        .env("BORGREPORT_COMPACT", "true")
        .assert()
        .stdout(predicate::str::contains(REPO).count(4))
        .stdout(predicate::str::contains("borg compact").count(1))
        .stdout(
            predicate::str::contains("Repositories with errors or warnings are not compacted.")
                .count(1),
        )
        .stdout(predicate::str::contains("- |").count(2));
}
