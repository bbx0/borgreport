// SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;

/// Response from of `borg info` command
#[derive(Deserialize, Clone, Debug)]
pub struct Info {
    pub archives: Vec<Archive>,
    pub cache: Cache,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Archive {
    pub hostname: String,
    pub name: String,
    #[serde(with = "borg_duration")]
    pub duration: std::time::Duration,
    pub start: jiff::civil::DateTime,
    pub stats: ArchiveStats,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ArchiveStats {
    pub original_size: i64,
    pub compressed_size: i64,
    pub deduplicated_size: i64,
    pub nfiles: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Cache {
    pub stats: CacheStats,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheStats {
    pub unique_csize: i64,
}

// borg duration is provided as a float value
mod borg_duration {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        std::time::Duration::try_from_secs_f64(f64::deserialize(deserializer)?)
            .map_err(<D::Error as serde::de::Error>::custom)
    }
}

mod tests {
    #[test]
    fn deserialize() {
        use crate::borg;
        let j = {
            r#"
        {
            "archives": [
                {
                    "chunker_params": [
                        "buzhash",
                        19,
                        23,
                        21,
                        4095
                    ],
                    "command_line": [
                        "/usr/bin/borg",
                        "create",
                        "::{utcnow}Z",
                        "tests.sh"
                    ],
                    "comment": "",
                    "duration": 0.014966,
                    "end": "2024-08-06T01:48:43.000000",
                    "hostname": "zen",
                    "id": "6081dae49103d30e49c0129010b9876e84854cd721d7b421cfeabb4140e1b79c",
                    "limits": {
                        "max_archive_size": 2.369885309471974e-05
                    },
                    "name": "2024-08-05T23:48:43Z",
                    "start": "2024-08-06T01:48:43.000000",
                    "stats": {
                        "compressed_size": 2408,
                        "deduplicated_size": 3100,
                        "nfiles": 1,
                        "original_size": 4489
                    },
                    "username": "phm"
                }
            ],
            "cache": {
                "path": "tests/borg/.cache/borg/cb666f9ab4737fb899b9f98b6fbc82d1afed27702b3702d21f761e420008b77a",
                "stats": {
                    "total_chunks": 3,
                    "total_csize": 2408,
                    "total_size": 4489,
                    "total_unique_chunks": 3,
                    "unique_csize": 3100,
                    "unique_size": 5149
                }
            },
            "encryption": {
                "mode": "repokey"
            },
            "repository": {
                "id": "cb666f9ab4737fb899b9f98b6fbc82d1afed27702b3702d21f761e420008b77a",
                "last_modified": "2024-08-06T01:48:43.000000",
                "location": "/tests/test3-checkok"
            }
        }
        "#
        };
        #[allow(clippy::unwrap_used)]
        let _ = serde_json::from_str::<borg::Info>(j).unwrap();
    }
}
