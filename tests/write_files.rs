// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Produce report files in folder target/tests.

mod common;
use common::*;
use std::sync::OnceLock;

/// Find the `target/tests/` folder.
/// On first use the folder is (re-)created to provide an empty directory.
fn target_dir() -> &'static std::path::Path {
    static TARGET_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
    TARGET_DIR.get_or_init(|| {
        let target_dir = std::path::Path::new(std::env::var_os("OUT_DIR").unwrap().as_os_str())
            .ancestors()
            .nth(4)
            .unwrap()
            .join("tests");
        let _ = std::fs::remove_dir_all(&target_dir);
        std::fs::create_dir_all(&target_dir).unwrap();
        target_dir
    })
}

/// Get the folder to store env files.
/// On first use the folder is created.
fn env_dir() -> &'static std::path::Path {
    static ENV_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
    ENV_DIR.get_or_init(|| {
        let env_dir = target_dir().join("env");
        std::fs::create_dir_all(&env_dir).unwrap();
        env_dir
    })
}

/// Create an env file for the repository
fn create_env<'a, I>(repo: &'a str, env: I)
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut repo_env = DEFAULT_ENV.to_vec();
    repo_env.push(("BORG_REPO", repo));
    repo_env.extend(env);

    std::fs::write(
        env_dir().join(format!("{repo}.env")),
        repo_env
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .unwrap();
}

/// Generate a report and write all formats into files.
/// - tests/repos/report.txt
/// - tests/repos/report.html
/// - tests/repos/report.metrics
///
/// This is not a `sealed_test` to keep the repos for inspection. Do not add another test into this module.
#[test]
#[ignore = "generating test repos for a combined report is slow."]
fn write_files() {
    // Change working directory to the tests/repos dir
    std::env::set_current_dir(target_dir()).unwrap();

    // Generate the test cases
    init::empty_directory("test1-no_init");
    create_env("test1-no_init", vec![]);

    init::no_archives("test2-no_backups");
    create_env("test2-no_backups", vec![]);

    init::one_archive("test3-check_ok", "{utcnow}Z");
    create_env("test3-check_ok", vec![]);

    init::faulty_archive("test4-check_not_ok", "{utcnow}Z");
    create_env("test4-check_not_ok", vec![]);

    init::two_archives("test5-two_archives_ok", "etc-{utcnow}Z", "srv-{utcnow}Z");
    create_env(
        "test5-two_archives_ok",
        vec![("BORGREPORT_GLOB_ARCHIVES", r#""etc-* srv-*""#)],
    );

    init::old_archive("test6-too_old_archive", "{utcnow}Z");
    create_env("test6-too_old_archive", vec![]);

    init::two_archives("test7-compact_ok", "etc-{utcnow}Z", "srv-{utcnow}Z");
    create_env(
        "test7-compact_ok",
        vec![
            ("BORGREPORT_GLOB_ARCHIVES", r#""etc-* srv-*""#),
            ("BORGREPORT_COMPACT", "true"),
            ("BORGREPORT_COMPACT_OPTIONS", r#""--threshold 0""#),
        ],
    );

    init::relocated("test8-relocated");
    create_env("test8-relocated", vec![]);

    cargo_bin()
        .env_clear()
        .args([
            "--env-dir",
            env_dir().to_str().unwrap(),
            "--text-to",
            "report.txt",
            "--html-to",
            "report.html",
            "--metrics-to",
            "report.metrics",
        ])
        .assert()
        .success();
}
