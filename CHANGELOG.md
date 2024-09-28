<!-- SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de> -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- markdownlint-configure-file {"MD024": { "siblings_only": true } } -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/bbx0/borgreport/commits/main/) - 2024-09-27

### Added

- Summarize status of BorgBackup repositories with statistics, warnings and error messages.
- Perform simple sanity checks
  - Warn about empty backup sources or repositories
  - Warn if the age of the last backup exceeds a threshold (24 hours by default)
- Send reports via Email
- Allow to run `borg check` as part of the report.
- Integrate with systemd as service and timer
