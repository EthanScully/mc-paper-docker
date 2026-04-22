use crate::{
    err::{Caller, ErrorCaller},
    utils,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaperVersionList {
    project_id: String,
    project_name: String,
    version_groups: Vec<String>,
    versions: Vec<String>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PaperBuildList {
    project_id: String,
    project_name: String,
    version: String,
    builds: Vec<u32>,
}

pub fn get_latest_version() -> utils::Result<String> {
    let paper_json = utils::get("https://api.papermc.io/v2/projects/paper").e()?;
    let paper_versions = serde_json::from_slice::<PaperVersionList>(&paper_json).e()?;
    let latest_version_string = paper_versions.versions.last().o()?.clone();
    Ok(latest_version_string)
}
pub fn get_latest_build(version: &str) -> utils::Result<u32> {
    let paper_json = utils::get(
        format!(
            "https://api.papermc.io/v2/projects/paper/versions/{}",
            version
        )
        .as_str(),
    )
    .e()?;
    let paper_versions = serde_json::from_slice::<PaperBuildList>(&paper_json).e()?;
    let latest_build = paper_versions.builds.last().o()?.clone();
    Ok(latest_build)
}
pub fn download_specified_version(version: &str, build: &str) -> utils::Result<Vec<u8>> {
    let download_url = format!(
        "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
        version, build, version, build
    );
    let download = utils::get(&download_url).e()?;
    Ok(download)
}
