use config::{Config, Environment, File};
use glob::glob;
use rat_theme4::{create_salsa_theme, palette::Palette, theme::SalsaTheme};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, sync::LazyLock};

use crate::{
    errors::Result,
    logging::{PROJECT_NAME, project_directory},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub(self) source_dirs: Vec<PathBuf>,
    pub(self) theme: Option<Theme>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Theme {
    InBuilt(String),
    Custom(Box<CustomTheme>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomTheme {
    pub name: String,
    pub theme: String,
    pub p: Palette,
}

impl From<CustomTheme> for SalsaTheme {
    fn from(c: CustomTheme) -> Self {
        let name = c.theme;
        let p = c.p;
        let mut theme = SalsaTheme::new(p);
        theme.name = name;
        theme
    }
}

impl From<Box<CustomTheme>> for SalsaTheme {
    fn from(c: Box<CustomTheme>) -> Self {
        (*c).into()
    }
}

impl From<Theme> for SalsaTheme {
    fn from(t: Theme) -> Self {
        match t {
            Theme::InBuilt(name) => create_salsa_theme(&name),
            Theme::Custom(c) => c.into(),
        }
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
    pub fn get_theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}
