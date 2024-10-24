<!-- SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de> -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- markdownlint-configure-file {"MD024": { "siblings_only": true } } -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/bbx0/borgreport/compare/v0.2.0...HEAD) - 2024-10-23

### Added

- [OpenMetrics](https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md) Text Exporter (Prometheus Metrics)
- Option `--metrics-to` to save metrics in application/openmetrics-text format

### Changed

- Replace tmpfiles.d configuration with systemd unit directives to manage the config and state directories

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
