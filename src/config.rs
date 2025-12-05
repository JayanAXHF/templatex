use std::{env, path::PathBuf, sync::LazyLock};

use config::{Config, Environment, File};
use glob::glob;
use serde::Deserialize;

use crate::{
    errors::Result,
    logging::{PROJECT_NAME, project_directory},
};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub(self) source_dirs: Vec<PathBuf>,
}

pub static CONFIG_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_CONFIG", &*PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});

pub(crate) fn get_config_dir() -> PathBuf {
    if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

impl Settings {
    pub fn new() -> Result<Self> {
        let s = Config::builder()
            .add_source(
                glob(
                    &get_config_dir()
                        .join("config")
                        .join("*")
                        .display()
                        .to_string(),
                )?
                .map(|p| File::from(p.expect("Failed to read config file")))
                .collect::<Vec<_>>(),
            )
            .add_source(
                Environment::with_prefix(&PROJECT_NAME)
                    .separator("__")
                    .prefix_separator("_"),
            );
        let s = s.build()?;
        Ok(s.try_deserialize()?)
    }
    pub fn get_source_dirs(&self) -> Vec<PathBuf> {
        self.source_dirs.clone()
    }
}
