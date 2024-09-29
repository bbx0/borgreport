<!-- SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de> -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# borgreport <!-- omit from toc -->

[![REUSE status](https://api.reuse.software/badge/github.com/bbx0/borgreport)](https://api.reuse.software/info/github.com/bbx0/borgreport)
[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg)](https://github.com/RichardLitt/standard-readme)

Summarize the status of multiple BorgBackup repositories in one report

This is a wrapper around [BorgBackup](https://borgbackup.readthedocs.io/en/stable/) to query the latest backup archives and perform health checks on repositories.

- Summarize status of BorgBackup repositories with statistics, warnings and error messages.
  - Save the report as file or send per mail.
- Perform simple sanity checks
  - Warn about empty backup sources or repositories
  - Warn if the age of the last backup exceeds a threshold (24 hours by default)
- Execute `borg check` as part of the report (optional).
- Plays nice as systemd service and timer.

## Table of Contents <!-- omit from toc -->

- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Example Report](#example-report)
- [Acknowledgments](#acknowledgments)
- [Contributing](#contributing)
- [License](#license)

## Install

- Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/borgreport) package.
- Debian packages are published in the [Releases](https://github.com/bbx0/borgreport/releases) section.
- Standalone binaries are published in the [Releases](https://github.com/bbx0/borgreport/releases) section.

## Usage

*borgreport* takes a directory with \*.env files as input. Each file must contain [environment variables](https://borgbackup.readthedocs.io/en/stable/usage/general.html#environment-variables) as understood by BorgBackup to access a repository. The filename will be shown as name of the repository in the report.

```bash
# Create an env file with the BORG_* variables for each repo
$ mkdir repos
$ cat repos/somerepo.env
BORG_REPO=/mnt/borg/repos/somerepo
BORG_PASSPHRASE=Secure

# Print the report to stdout and run `borg check` against the repos
borgreport --env-dir repos --check

# Send report via `sendmail` to admin@host.invalid
borgreport --env-dir repos --mail-to admin@example.com
```

The [systemd unit](assets/systemd/) expects the *.env files in folder `/etc/borgreport/repos` or in `~/.config/borgreport/repos` when run as user unit.

## Configuration

The \*.env file can contain additional `BORGREPORT_*` variables to change the report.

Please check the [man page](https://html-preview.github.io/?url=https://github.com/bbx0/borgreport/blob/main/assets/man/borgreport.html) for all available options or run `man 1 borgreport`.

```bash
# A list of space separated archive globs to include multiple archives per repository. (Default: "")
# Example: "etc-* srv-*" for archive names starting with etc- or srv-.
BORGREPORT_GLOB_ARCHIVES=<GLOB>
# Enables the execution of ‘borg check‘. (Default: false)
BORGREPORT_CHECK=<true|false>
# Threshold to warn, when the last backup is older than <HOURS>. (Default: 24)
BORGREPORT_MAX_AGE_HOURS=<HOURS>
```

`BORGREPORT_*` variables are interpreted in the following sequence overruling previous values.

1) Environment variable passed directly to *borgreport*
1) Repository configuration as read from the \*.env file
1) Command line argument passed to *borgreport* (if applicable)

## Example Report

```text
==== Backup report (2024-09-29) ====

=== Errors ===

  * data1: Data integrity error: Invalid segment entry header [segment 0, offset 530]: unpack requires a buffer of 9 bytes
   Finished full repository check, errors found.

=== Warnings ===

  * web1: Repository is empty

=== Summary ===

| Repository          | Hostname | Last archive             | Start      | Duration | Source | Δ Archive | ∑ Repository |
|---------------------|----------|--------------------------|------------|----------|--------|-----------|--------------|
| web1                |          |                          | 0000-01-01 |      0ns |     0B |        0B |           0B |
| web2                | host2    | 2024-09-29T14:19:43Z     | 2024-09-29 |   17.1ms |  5.3kB |       3kB |          3kB |
| data1               | host2    | 2024-09-29T14:19:44Z     | 2024-09-29 |     15ms |  5.3kB |       3kB |          3kB |
| media               | host3    | etc-2024-09-29T14:19:45Z | 2024-09-29 |   14.4ms |  5.3kB |      505B |        3.5kB |
| media               | host3    | srv-2024-09-29T14:19:45Z | 2024-09-29 |    2.5ms |  5.3kB |      503B |        3.5kB |

=== `borg check` result ===

| Repository          | Archive                  | Duration | Okay |
|---------------------|--------------------------|----------|------|
| web1                |                          |  223.4ms |  yes |
| web2                | 2024-09-29T14:19:43Z     |     6.7s |  yes |
| data1               | 2024-09-29T14:19:44Z     |   10.38s |   no |
| media               | etc-2024-09-29T14:19:45Z |    4.53s |  yes |
| media               | srv-2024-09-29T14:19:45Z |  14:02.2 |  yes |

Generated Sun, 29 Sep 2024 16:19:48 +0200 (borgreport 0.1.0)
```

## Acknowledgments

- [BorgBackup](https://github.com/borgbackup/borg) the deduplicating archiver with compression and authenticated encryption.
- *borgreport* is inspired by the status report feature of [rsbackup](https://github.com/ewxrjk/rsbackup). A brilliant orchestrator for `rsync`-based backups.

## Contributing

Please feel free to [open an issue](https://github.com/bbx0/borgreport/issues/new) at GitHub.

## License

Copyright © 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>

This project conforms to the [REUSE Specification](https://reuse.software/spec/), where each file contains a comment header with licensing information. See [REUSE.toml](REUSE.toml) for exceptions.

The *borgreport* source code is distributed under the [GNU General Public License v3.0 or later](LICENSES/GPL-3.0-or-later.txt). Part of the documentation is licensed under the [Creative Commons Attribution Share Alike 4.0 International](LICENSES/CC-BY-SA-4.0.txt) license and some files are licensed under the [Creative Commons Zero v1.0 Universal](LICENSES/CC0-1.0.txt) license.

The *borgreport* source code does not bundle any third-party libraries, but third-party libraries are statically linked into the binary distribution. See [LICENSE-THIRD-PARTY.md](LICENSE-THIRD-PARTY.md) for details.
