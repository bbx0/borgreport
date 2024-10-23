// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Formatter;
use crate::{borg::BORG_TZ, report::Report};
use prometheus_client::{
    collector::Collector,
    encoding::{text::encode, DescriptorEncoder, EncodeLabelSet, EncodeMetric},
    metrics::{
        family::Family,
        gauge::{ConstGauge, Gauge},
        info::Info,
    },
    registry::{Registry, Unit},
};

/// A metric label set: `repository`
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct RepositoryLabel {
    repository: String,
}
impl From<String> for RepositoryLabel {
    fn from(value: String) -> Self {
        Self { repository: value }
    }
}

/// A metric label set: `repository`, `hostname` and `archive_glob`
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ArchiveGlobHostnameLabel {
    repository: String,
    hostname: String,
    archive_glob: Option<String>,
}
impl From<(String, String, Option<String>)> for ArchiveGlobHostnameLabel {
    fn from(value: (String, String, Option<String>)) -> Self {
        let (repository, hostname, archive_glob) = value;
        Self {
            repository,
            hostname,
            archive_glob,
        }
    }
}

/// A metric label set: `repository` and `archive_glob`
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ArchiveGlobLabel {
    repository: String,
    archive_glob: Option<String>,
}
impl From<(String, Option<String>)> for ArchiveGlobLabel {
    fn from(value: (String, Option<String>)) -> Self {
        let (repository, archive_glob) = value;
        Self {
            repository,
            archive_glob,
        }
    }
}

/// Round the `duration` up to whole seconds
fn duration_as_secs(duration: std::time::Duration) -> Result<i64, std::fmt::Error> {
    jiff::Span::try_from(jiff::SignedDuration::from_secs_f64(duration.as_secs_f64()))
        .and_then(|span| {
            span.round(
                jiff::SpanRound::new()
                    .smallest(jiff::Unit::Second)
                    .mode(jiff::RoundMode::Expand),
            )
            .map(|span| span.get_seconds())
        })
        .map_err(|_| std::fmt::Error)
}

/// Collect metrics from the `Report` meta structure.
#[derive(Debug, Default)]
struct ReportCollector {
    // Repository metrics
    unique_csize: Family<RepositoryLabel, Gauge>,

    // Metrics of the last archive (`borg create`)
    create_start_timestamp: Family<ArchiveGlobHostnameLabel, Gauge>,
    create_duration: Family<ArchiveGlobHostnameLabel, Gauge>,
    create_original_size: Family<ArchiveGlobHostnameLabel, Gauge>,
    create_compressed_size: Family<ArchiveGlobHostnameLabel, Gauge>,
    create_deduplicated_size: Family<ArchiveGlobHostnameLabel, Gauge>,
    create_nfiles: Family<ArchiveGlobHostnameLabel, Gauge>,

    // Metrics of the check of the last archive (`borg check`)
    check_duration: Family<ArchiveGlobLabel, Gauge>,
    check_success: Family<ArchiveGlobLabel, Gauge>,
}

impl Collector for ReportCollector {
    /// Write annotated metrics into the registry
    fn encode(&self, mut encoder: DescriptorEncoder) -> Result<(), std::fmt::Error> {
        let Self {
            unique_csize,
            create_original_size,
            create_compressed_size,
            create_deduplicated_size,
            create_nfiles,
            create_start_timestamp,
            create_duration,
            check_duration,
            check_success,
        } = self;

        /// Encode a metric with the a unit
        macro_rules! register_with_unit {
            ($metric:ident, $name:literal, $unit:path, $help:literal) => {
                $metric.encode(encoder.encode_descriptor(
                    $name,
                    $help,
                    Some(&$unit),
                    self.$metric.metric_type(),
                )?)?;
            };
        }
        register_with_unit!(
            unique_csize,
            "deduplicated_compressed_size",
            Unit::Bytes,
            "Size of the backup repository in bytes (compressed and deduplicated)"
        );
        register_with_unit!(
            create_original_size,
            "create_last_original_size",
            Unit::Bytes,
            "Source size of the last backup archive in bytes"
        );
        register_with_unit!(
            create_compressed_size,
            "create_last_compressed_size",
            Unit::Bytes,
            "Compressed size of the last backup archive in bytes (not deduplicated)"
        );
        register_with_unit!(
            create_deduplicated_size,
            "create_last_deduplicated_compressed_size",
            Unit::Bytes,
            "Deduplicated and compressed size of the last backup archive in bytes"
        );
        register_with_unit!(
            create_start_timestamp,
            "create_last_start_timestamp",
            Unit::Seconds,
            "Unix time when the last backup was started"
        );
        register_with_unit!(
            create_duration,
            "create_last_duration",
            Unit::Seconds,
            "Duration of the last backup in seconds"
        );

        create_nfiles.encode(encoder.encode_descriptor(
            "create_last_files",
            "Number of files in the last archive",
            None,
            create_nfiles.metric_type(),
        )?)?;

        register_with_unit!(
            check_duration,
            "check_last_duration",
            Unit::Seconds,
            "Duration of the check of the last archive in seconds"
        );

        let boolean = Unit::Other("boolean".to_string());
        register_with_unit!(
            check_success,
            "check_last_success",
            boolean,
            "True (1) if the check of the last archive was successful"
        );

        Ok(())
    }
}

impl From<&Report> for ReportCollector {
    /// Convert a `Report` into metrics
    ///
    /// A `Report` is a representation for humans. Empty data (or a value of 0)
    /// in the `Report` can translate to no actual measurement (no metric).
    fn from(report: &Report) -> Self {
        let Self {
            unique_csize,
            create_original_size,
            create_compressed_size,
            create_deduplicated_size,
            create_nfiles,
            create_start_timestamp,
            create_duration,
            check_duration,
            check_success,
        } = Self::default();

        // Process the summary table.
        for archive in &*report.summary {
            let repository_label = &RepositoryLabel::from(archive.repository.clone());
            let archive_label = &ArchiveGlobHostnameLabel::from((
                archive.repository.clone(),
                archive.hostname.clone(),
                archive.archive_glob.clone(),
            ));

            // Ok: The size of the repo can be zero.
            unique_csize
                .get_or_create(repository_label)
                .set(archive.unique_csize);

            // Skip all entries without an archive name since there was no last archive created.
            if !&archive.archive.is_empty() {
                create_original_size
                    .get_or_create(archive_label)
                    .set(archive.original_size);
                create_compressed_size
                    .get_or_create(archive_label)
                    .set(archive.compressed_size);
                create_deduplicated_size
                    .get_or_create(archive_label)
                    .set(archive.deduplicated_size);
                create_nfiles
                    .get_or_create(archive_label)
                    .set(archive.nfiles);

                // Only create a `last_start_timestamp` if it is a valid non-zero Unix time
                if let Some(start) = archive
                    .start
                    .intz(BORG_TZ)
                    .ok()
                    .map(|t| t.timestamp().as_second())
                    .filter(|t| *t > 0)
                {
                    create_start_timestamp
                        .get_or_create(archive_label)
                        .set(start);
                }

                if let Ok(duration) = duration_as_secs(archive.duration) {
                    create_duration.get_or_create(archive_label).set(duration);
                }
            }
        }

        // Process `borg check` results
        for check in &*report.checks {
            let archive_label =
                &ArchiveGlobLabel::from((check.repository.clone(), check.archive_glob.clone()));

            if let Ok(duration_secs) = duration_as_secs(check.duration) {
                check_duration
                    .get_or_create(archive_label)
                    .set(duration_secs);
            }

            check_success
                .get_or_create(archive_label)
                .set(check.status.success().into());
        }

        Self {
            unique_csize,
            create_start_timestamp,
            create_duration,
            create_original_size,
            create_compressed_size,
            create_deduplicated_size,
            create_nfiles,
            check_duration,
            check_success,
        }
    }
}

/// Metrics `Formatter` (application/openmetrics-text)
pub struct Metrics;
impl Formatter<Report> for Metrics {
    fn format(buf: &mut String, report: &Report) -> std::fmt::Result {
        let mut registry = <Registry>::default();

        //borgreport info metadata and generated at timestamp
        registry.register(
            env!("CARGO_PKG_NAME"),
            "borgreport metadata",
            Info::new([
                ("name", env!("CARGO_PKG_NAME")),
                ("version", env!("CARGO_PKG_VERSION")),
            ]),
        );
        registry.register_with_unit(
            concat!(env!("CARGO_PKG_NAME"), "_last_report_timestamp"),
            "Unix time when the metrics were generated",
            Unit::Seconds,
            ConstGauge::new(jiff::Timestamp::now().as_second()),
        );

        // Collect metrics from the `Report`
        let borg_registry = registry.sub_registry_with_prefix("borg");
        borg_registry.register_collector(Box::new(ReportCollector::from(report)));

        encode(buf, &registry)?;
        Ok(())
    }
}