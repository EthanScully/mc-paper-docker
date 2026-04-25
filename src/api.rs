use std::{collections::HashMap, *};

use crate::{err::ErrorCaller, utils};
// API JSON STRUCTS //
#[derive(Debug, serde::Deserialize)]
pub struct PaperVersionList {
    versions: HashMap<String, Vec<String>>,
}
#[derive(Debug, serde::Deserialize)]
struct PaperBuild {
    id: u16,
    channel: String,
    downloads: PaperBuildDownload,
}
#[derive(Debug, serde::Deserialize)]
struct PaperBuildDownload {
    #[serde(rename = "server:default")]
    server_default: PaperBuildDownloadDefault,
}
#[derive(Debug, serde::Deserialize)]
struct PaperBuildDownloadDefault {
    url: String,
}
//                  //
struct VersionList(Vec<Version>);
impl VersionList {
    fn get() -> utils::Result<Self> {
        let json_raw = utils::get("https://fill.papermc.io/v3/projects/paper").e()?;
        let json_list = serde_json::from_slice::<PaperVersionList>(&json_raw).e()?;
        let mut versions = Vec::<Version>::new();
        for (_, vs) in json_list.versions {
            for v in vs {
                let version = Version::new(v);
                versions.push(version);
            }
        }
        let mut versions = VersionList(versions);
        versions.0.sort();
        versions.0.reverse();
        Ok(versions)
    }
}
struct Version {
    value: (u16, u16, u16),
    original: String,
    builds: Option<BuildList>,
}
impl Version {
    fn new(original: String) -> Self {
        let original_split = original.split(".").collect::<Vec<_>>();
        let index_2 = if original_split.len() > 2 {
            original_split[2].parse::<u16>().unwrap_or_default()
        } else {
            0
        };
        let index_1 = if original_split.len() > 1 {
            original_split[1].parse::<u16>().unwrap_or_default()
        } else {
            0
        };
        let index_0 = if original_split.len() > 0 {
            original_split[0].parse::<u16>().unwrap_or_default()
        } else {
            0
        };
        Self {
            value: (index_0, index_1, index_2),
            original,
            builds: None,
        }
    }
    fn import(value: (u16, u16, u16)) -> Self {
        let original = if value.2 == 0 {
            format!("{}.{}", value.0, value.1)
        } else {
            format!("{}.{}.{}", value.0, value.1, value.2)
        };
        Self {
            value,
            original,
            builds: None,
        }
    }
    fn pull(&mut self) {}
}
impl Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let cmp_value_0 = self.value.0.cmp(&other.value.0);
        if cmp_value_0 != cmp::Ordering::Equal {
            return cmp_value_0;
        }
        let cmp_value_1 = self.value.1.cmp(&other.value.1);
        if cmp_value_1 != cmp::Ordering::Equal {
            return cmp_value_1;
        }
        self.value.2.cmp(&other.value.2)
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}
impl Eq for Version {}
struct BuildList(Vec<BuildInfo>);
impl BuildList {
    fn get(version: &str) -> utils::Result<Self> {
        let json_raw = utils::get(&format!(
            "https://fill.papermc.io/v3/projects/paper/versions/{}/builds",
            version
        ))
        .e()?;
        let json_list = serde_json::from_slice::<Vec<PaperBuild>>(&json_raw).e()?;
        let mut builds = Vec::<BuildInfo>::new();
        for b in json_list {
            builds.push(BuildInfo::new(&b));
        }
        let mut builds = BuildList(builds);
        builds.0.sort();
        builds.0.reverse();
        Ok(builds)
    }
}
struct BuildInfo {
    id: u16,
    channel: BuildChannel,
    url: String,
}
impl BuildInfo {
    fn new(build: &PaperBuild) -> Self {
        BuildInfo {
            id: build.id,
            channel: BuildChannel::parse(&build.channel),
            url: build.downloads.server_default.url.clone(),
        }
    }
}
impl Ord for BuildInfo {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl PartialOrd for BuildInfo {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for BuildInfo {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}
impl Eq for BuildInfo {}
enum BuildChannel {
    ALPHA,
    BETA,
    STABLE,
    UNKNOWN,
}
impl BuildChannel {
    fn parse(channel: &str) -> BuildChannel {
        match channel {
            "ALPHA" => BuildChannel::ALPHA,
            "BETA" => BuildChannel::BETA,
            "STABLE" => BuildChannel::STABLE,
            _ => BuildChannel::UNKNOWN,
        }
    }
}
