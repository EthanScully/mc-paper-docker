use crate::{
    api,
    err::{Caller, ErrorCaller},
    utils,
};

use std::{path::Path, *};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MinecraftState {
    version: (u16, u16, u16),
    build: u16,
    #[serde(skip)]
    current_build: Option<api::Build>,
}
impl MinecraftState {
    pub fn write_json(self, path: &Path) -> utils::Result<()> {
        let raw_data = serde_json::to_vec_pretty(&self).e()?;
        fs::write(path, raw_data).e()?;
        Ok(())
    }
    // return new mcstate if needs update
    pub fn need_update(&mut self) -> utils::Result<Option<Self>> {
        let mut version_list = api::VersionList::get().e()?;
        if let Some(latest_build) = version_list.latest_stable_build().e()? {
            let current_build = if self.current_build.is_some() {
                self.current_build.take().unwrap()
            } else {
                api::Build::import(self.build, api::Version::import(self.version))
            };
            if latest_build > current_build {
                return Ok(Some(Self {
                    version: latest_build.version.value,
                    build: latest_build.id,
                    current_build: Some(latest_build),
                }));
            } else {
                self.current_build = Some(current_build);
            }
        };
        return Ok(None);
    }
    pub fn download(&self) -> utils::Result<Vec<u8>> {
        let url = self.current_build.as_ref().o()?.url.clone();
        let data = utils::get(&url).e()?;
        Ok(data)
    }
}
pub fn load_json(path: &Path) -> utils::Result<MinecraftState> {
    if !path.exists() {
        return Ok(MinecraftState {
            version: (0, 0, 0),
            build: 0,
            current_build: None,
        });
    }
    let file_data = fs::read(path).e()?;
    Ok(serde_json::from_slice::<MinecraftState>(&file_data).e()?)
}
