<!-- SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de> -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# borgreport <!-- omit from toc -->

[![AUR package](https://repology.org/badge/version-for-repo/aur/borgreport.svg)](https://aur.archlinux.org/packages/borgreport)
[![Crates.io Version](https://img.shields.io/crates/v/borgreport?color=brightgreen)](https://crates.io/crates/borgreport)
[![REUSE status](https://api.reuse.software/badge/github.com/bbx0/borgreport)](https://api.reuse.software/info/github.com/bbx0/borgreport)
[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg)](https://github.com/RichardLitt/standard-readme)

Summarize the status of multiple BorgBackup repositories in one report and export metrics

This is a wrapper around [BorgBackup](https://borgbackup.readthedocs.io/en/stable/) to query the latest backup archives and perform health checks on repositories.

- Summarize status of BorgBackup repositories with statistics, warnings and error messages.
  - Save the report as file or send per mail.
  - Export [OpenMetrics](https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md) (Prometheus Metrics) for the last archive.
- Perform simple sanity checks
  - Warn about empty backup sources or repositories
  - Warn if the age of the last backup exceeds a threshold (24 hours by default)
- Execute `borg check` as part of the report (optional).
- Plays nice as systemd service and timer.

## Table of Contents <!-- omit from toc -->

- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Example](#example)
  - [Report](#report)
  - [Metrics](#metrics)
- [Acknowledgments](#acknowledgments)
- [Contributing](#contributing)
- [License](#license)

## Install

- Arch Linux users can use the [AUR](https://aur.archlinux.org/packages/borgreport) package.
- Debian packages are published in the [Releases](https://github.com/bbx0/borgreport/releases) section.
- Standalone binaries are published in the [Releases](https://github.com/bbx0/borgreport/releases) section
- Install standalone binaries with [cargo binstall](https://github.com/cargo-bins/cargo-binstall) `cargo binstall borgreport`
- Compile from source via [cargo](https://doc.rust-lang.org/cargo/) `cargo install borgreport`

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

# Send the report via `sendmail` to admin@host.invalid
borgreport --env-dir repos --mail-to admin@example.com

# Write the metrics to file borg.metrics and print a text report to stdout
borgreport --env-dir repos --metrics-to borg.metrics --text-to=-
```

*borgreport* can inherit BORG_* env vars for a single repository. This allows to run `borgreport` after `borg` while reusing the environment.

```bash
export BORG_REPO=/mnt/borg/repos/somerepo
export BORG_PASSPHRASE=Secure

# Run borg as normal
borg create borg create '::{utcnow}' /data

# Export the metrics for most recent archive to file borg.metrics
borgreport --env-inherit somerepo --metrics-to borg.metrics

# Export the metrics for archives starting with etc- or srv- to file borg.metrics
borgreport --env-inherit somerepo --glob-archives 'etc-* srv-*' --metrics-to borg.metrics
```

The [systemd unit](assets/systemd/):

- expects the *.env files in folder `/etc/borgreport/repos` or in `~/.config/borgreport/repos` when run as user unit
- writes metrics to `/var/lib/borgreport/metrics` or to `~/.local/state/borgreport/metrics` when run as user unit

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

## Example

### Report

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

### Metrics

```ini
# HELP borgreport borgreport metadata.
# TYPE borgreport info
borgreport_info{name="borgreport",version="0.3.0"} 1
# HELP borgreport_last_report_timestamp_seconds Unix time when the metrics were generated.
# TYPE borgreport_last_report_timestamp_seconds gauge
# UNIT borgreport_last_report_timestamp_seconds seconds
borgreport_last_report_timestamp_seconds 1729761766
# HELP borg_deduplicated_compressed_size_bytes Size of the backup repository in bytes (compressed and deduplicated)
# TYPE borg_deduplicated_compressed_size_bytes gauge
# UNIT borg_deduplicated_compressed_size_bytes bytes
borg_deduplicated_compressed_size_bytes{repository="web2"} 3041
borg_deduplicated_compressed_size_bytes{repository="media"} 3551
# HELP borg_create_last_original_size_bytes Source size of the last backup archive in bytes
# TYPE borg_create_last_original_size_bytes gauge
# UNIT borg_create_last_original_size_bytes bytes
borg_create_last_original_size_bytes{repository="web2",hostname="host2",archive_glob=""} 5292
borg_create_last_original_size_bytes{repository="media",hostname="host3",archive_glob="etc-*"} 5292
borg_create_last_original_size_bytes{repository="media",hostname="host3",archive_glob="srv-*"} 5292
# HELP borg_create_last_compressed_size_bytes Compressed size of the last backup archive in bytes (not deduplicated)
# TYPE borg_create_last_compressed_size_bytes gauge
# UNIT borg_create_last_compressed_size_bytes bytes
borg_create_last_compressed_size_bytes{repository="web2",hostname="host2",archive_glob=""} 2349
borg_create_last_compressed_size_bytes{repository="media",hostname="host3",archive_glob="etc-*"} 2349
borg_create_last_compressed_size_bytes{repository="media",hostname="host3",archive_glob="srv-*"} 2349
# HELP borg_create_last_deduplicated_compressed_size_bytes Deduplicated and compressed size of the last backup archive in bytes
# TYPE borg_create_last_deduplicated_compressed_size_bytes gauge
# UNIT borg_create_last_deduplicated_compressed_size_bytes bytes
borg_create_last_deduplicated_compressed_size_bytes{repository="web2",hostname="host2",archive_glob=""} 3041
borg_create_last_deduplicated_compressed_size_bytes{repository="media",hostname="host3",archive_glob="srv-*"} 503
borg_create_last_deduplicated_compressed_size_bytes{repository="media",hostname="host3",archive_glob="etc-*"} 504
# HELP borg_create_last_start_timestamp_seconds Unix time when the last backup was started
# TYPE borg_create_last_start_timestamp_seconds gauge
# UNIT borg_create_last_start_timestamp_seconds seconds
borg_create_last_start_timestamp_seconds{repository="web2",hostname="host2",archive_glob=""} 1729761635
borg_create_last_start_timestamp_seconds{repository="media",hostname="host3",archive_glob="etc-*"} 1729761636
borg_create_last_start_timestamp_seconds{repository="media",hostname="host3",archive_glob="srv-*"} 1729761636
# HELP borg_create_last_duration_seconds Duration of the last backup in seconds
# TYPE borg_create_last_duration_seconds gauge
# UNIT borg_create_last_duration_seconds seconds
borg_create_last_duration_seconds{repository="web2",hostname="host2",archive_glob=""} 1
borg_create_last_duration_seconds{repository="media",hostname="host3",archive_glob="etc-*"} 1
borg_create_last_duration_seconds{repository="media",hostname="host3",archive_glob="srv-*"} 1
# HELP borg_create_last_files Number of files in the last archive
# TYPE borg_create_last_files gauge
borg_create_last_files{repository="web2",hostname="host2",archive_glob=""} 1
borg_create_last_files{repository="media",hostname="host3",archive_glob="srv-*"} 1
borg_create_last_files{repository="media",hostname="host3",archive_glob="etc-*"} 1
# HELP borg_check_last_duration_seconds Duration of the check of the last archive in seconds
# TYPE borg_check_last_duration_seconds gauge
# UNIT borg_check_last_duration_seconds seconds
borg_check_last_duration_seconds{repository="web2",archive_glob=""} 1
borg_check_last_duration_seconds{repository="media",archive_glob="etc-*"} 1
borg_check_last_duration_seconds{repository="media",archive_glob="srv-*"} 1
# HELP borg_check_last_success_boolean True (1) if the check of the last archive was successful
# TYPE borg_check_last_success_boolean gauge
# UNIT borg_check_last_success_boolean boolean
borg_check_last_success_boolean{repository="web2",archive_glob=""} 1
borg_check_last_success_boolean{repository="media",archive_glob="etc-*"} 1
borg_check_last_success_boolean{repository="media",archive_glob="srv-*"} 1
# EOF
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
