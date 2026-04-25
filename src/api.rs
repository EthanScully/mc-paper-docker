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
#[derive(Debug)]
pub struct VersionList(Vec<Version>);
impl VersionList {
    pub fn get() -> utils::Result<Self> {
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
    pub fn latest_stable_build(&mut self) -> utils::Result<Option<Build>> {
        for version in &mut self.0 {
            if version.builds.is_none() {
                version.pull_builds().e()?
            }
            let latest_stable_build = version.builds.as_ref().unwrap().latest_stable();
            if latest_stable_build.is_none() {
                continue;
            }
            return Ok(latest_stable_build);
        }
        return Ok(None);
    }
}
#[derive(Debug)]
pub struct Version {
    pub value: (u16, u16, u16),
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
    pub fn import(value: (u16, u16, u16)) -> Self {
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
    fn pull_builds(&mut self) -> utils::Result<()> {
        self.builds = Some(BuildList::get(&self).e()?);
        Ok(())
    }
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
impl Clone for Version {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            original: self.original.clone(),
            builds: None,
        }
    }
}
#[derive(Debug)]
struct BuildList(Vec<Build>);
impl BuildList {
    fn get(version: &Version) -> utils::Result<Self> {
        let json_raw = utils::get(&format!(
            "https://fill.papermc.io/v3/projects/paper/versions/{}/builds",
            version.original
        ))
        .e()?;
        let json_list = serde_json::from_slice::<Vec<PaperBuild>>(&json_raw).e()?;
        let mut builds = Vec::<Build>::new();
        for b in json_list {
            builds.push(Build::new(&b, version));
        }
        let mut builds = BuildList(builds);
        builds.0.sort();
        builds.0.reverse();
        Ok(builds)
    }
    fn latest_stable(&self) -> Option<Build> {
        for build in &self.0 {
            if build.channel == BuildChannel::STABLE {
                return Some(build.clone());
            }
        }
        return None;
    }
}
#[derive(Clone, Debug)]
pub struct Build {
    pub version: Version,
    pub id: u16,
    channel: BuildChannel,
    pub url: String,
}
impl Build {
    fn new(build: &PaperBuild, version: &Version) -> Self {
        Build {
            version: version.clone(),
            id: build.id,
            channel: BuildChannel::parse(&build.channel),
            url: build.downloads.server_default.url.clone(),
        }
    }
    pub fn import(id: u16, version: Version) -> Self {
        Self {
            version,
            id,
            channel: BuildChannel::UNKNOWN,
            url: String::new(),
        }
    }
}
impl Ord for Build {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let c = self.version.cmp(&other.version);
        if c == cmp::Ordering::Equal {
            self.id.cmp(&other.id)
        } else {
            c
        }
    }
}
impl PartialOrd for Build {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Build {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}
impl Eq for Build {}
#[derive(PartialEq, Clone, Debug)]
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
