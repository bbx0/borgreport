<!-- SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de> -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- markdownlint-configure-file {"MD024": { "siblings_only": true } } -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/bbx0/borgreport/compare/v0.3.0...HEAD) - 2025-02-22

### Changed

- Use rust edition 2024 and raise MSRV to 1.85
- Update dependencies
- Refactor to use `jiff::SignedDuration`
- Adopt new ignored files from REUSE specification 3.3

## [0.3.0](https://github.com/bbx0/borgreport/compare/v0.2.0...v0.3.0) - 2024-11-12

### Added

- [OpenMetrics](https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md) Text Exporter (Prometheus Metrics)
- Option `--metrics-to` to save metrics in `application/openmetrics-text` format
- Option `--env-inherit` to inherit BORG_* variables for a single repository from the active environment. This allows to run `borgreport` after `borg` while reusing the environment.
- Options `--text-to` and `--html-to` to save the report in `text/plain` or  `text/html` format
- Option `--check-options` to supply raw `borg check` options, when `--check` is enabled.

### Changed

- Replaced the tmpfiles.d configuration with systemd unit directives to manage the config and state directories.
- When called without `--env-inherit` and `--env-dir` but a `BORG_REPO` environment variable is provided, the repository name is set to the final component of the `BORG_REPO` path. This is a convenience feature. It is recommended to use `--env-inherit` to name the repository explicitly or to use a `--env-dir`.

### Removed

- The options `--file-to` and `--file-format` are removed and replaced by individual flags: `--text-to`, `--html-to`.

## [0.2.0](https://github.com/bbx0/borgreport/compare/v0.1.0...v0.2.0) - 2024-10-13

### Added

- Support to format the report in HTML
- Option `--file-format` to choose the output file format

### Changed

- Use `--bypass-lock` for the `borg info` command
- Send Emails as multipart with `text/plain` and `text/html`

## [0.1.0](https://github.com/bbx0/borgreport/releases/tag/v0.1.0) - 2024-10-02

### Added

- Summarize status of BorgBackup repositories with statistics, warnings and error messages.
- Perform simple sanity checks
  - Warn about empty backup sources or repositories
  - Warn if the age of the last backup exceeds a threshold (24 hours by default)
- Send reports via Email
- Allow to run `borg check` as part of the report.
- Integrate with systemd as service and timer
