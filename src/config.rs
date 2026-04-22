use crate::{
    api,
    err::{self, ErrorCaller},
    utils,
};

use std::{path::Path, *};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MinecraftState {
    version: (u32, u32, u32),
    build: u32,
}
impl MinecraftState {
    pub fn write_json(self, path: &Path) -> utils::Result<()> {
        let raw_data = serde_json::to_vec_pretty(&self).e()?;
        fs::write(path, raw_data).e()?;
        Ok(())
    }
    fn new(version: (u32, u32, u32), build: u32) -> Self {
        Self { version, build }
    }
    // return true if an update is needed
    pub fn check_update(&self) -> utils::Result<Option<Self>> {
        let latest_version = create_latest_version().e()?;

        if self.compare(&latest_version) {
            Ok(Some(latest_version))
        } else {
            Ok(None)
        }
    }
    // return true if compare is larger
    fn compare(&self, compare: &Self) -> bool {
        if self.version.0 < compare.version.0 {
            return true;
        }
        if self.version.1 < compare.version.1 {
            return true;
        }
        if self.version.2 < compare.version.2 {
            return true;
        }
        if self.build < compare.build {
            return true;
        }
        false
    }
    pub fn download(&self) -> utils::Result<Vec<u8>> {
        let version_string = self.version_tuple_to_str();
        Ok(
            api::download_specified_version(&version_string, format!("{}", self.build).as_str())
                .e()?,
        )
    }
    fn version_tuple_to_str(&self) -> String {
        if self.version.2 == 0 {
            format!("{}.{}", self.version.0, self.version.1)
        } else {
            format!("{}.{}.{}", self.version.0, self.version.1, self.version.2)
        }
    }
}
pub fn load_json(path: &Path) -> utils::Result<MinecraftState> {
    if !path.exists() {
        return Ok(MinecraftState {
            version: (0, 0, 0),
            build: 0,
        });
    }
    let file_data = fs::read(path).e()?;
    Ok(serde_json::from_slice::<MinecraftState>(&file_data).e()?)
}
fn create_latest_version() -> utils::Result<MinecraftState> {
    let latest_version_string = api::get_latest_version().e()?;
    let latest_build = api::get_latest_build(&latest_version_string).e()?;
    let latest_version = version_str_to_tuple(&latest_version_string).e()?;
    Ok(MinecraftState::new(latest_version, latest_build))
}

fn version_str_to_tuple(s: &str) -> utils::Result<(u32, u32, u32)> {
    let split = s.split(".").collect::<Vec<_>>();
    if split.len() != 2 && split.len() != 3 {
        return Err(err::new(format!("unexpected result: {}", s)))?;
    }
    let a = split[0].parse::<u32>().e()?;
    let b = split[1].parse::<u32>().e()?;
    let c = if split.len() == 3 {
        split[2].parse::<u32>().e()?
    } else {
        0
    };
    Ok((a, b, c))
}
