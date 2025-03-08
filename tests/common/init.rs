// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Helper to create repositories for test cases

use super::*;

/// An empty directory but no initialized repo
pub fn empty_directory(dir: &str) {
    std::fs::create_dir(dir).unwrap();
}

/// A repository without backup archives
pub fn no_archives(repo: &str) {
    borg_init(repo);
}

/// A repository with one backup archive
pub fn one_archive(repo: &str, name: &str) {
    borg_init(repo);
    borg_create(repo, name, []);
}

/// A repository with two backup archives
pub fn two_archives(repo: &str, name1: &str, name2: &str) {
    borg_init(repo);
    borg_create(repo, name1, []);
    borg_create(repo, name2, []);
}

/// A repository with a faulty backup archive
pub fn faulty_archive(repo: &str, name: &str) {
    one_archive(repo, name);
    // tamper with the repo data
    use std::io::Write;
    let mut file = std::fs::File::options()
        .append(true)
        .open(format!("{repo}{DS}data{DS}0{DS}0"))
        .unwrap();
    writeln!(file, "FOOBAR").unwrap();
}

/// A repository with one backup archive created at 1970-01-02T00:00:00Z
pub fn old_archive(repo: &str, name: &str) {
    borg_init(repo);
    borg_create(repo, name, ["--timestamp", "1970-01-02T00:00:00"]);
}
