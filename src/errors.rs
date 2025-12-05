use derive_more::Display;
use thiserror::Error;

pub type Result<T> = color_eyre::Result<T, Error>;

#[derive(Error, Debug, Display)]
pub enum Error {
    ParsingError,
    TeraError(#[from] tera::Error),
    IoError(#[from] std::io::Error),
    InputError(#[from] InputError),
    PatternError(#[from] glob::PatternError),
    ConfigError(#[from] config::ConfigError),
    TemplateConfigError(#[from] toml::de::Error),
    Other(#[from] color_eyre::Report),
}

#[derive(Error, Debug)]
#[error("Invalid input for field {0}. This may be due to a typo or a missing value.")]
pub struct InputError(String);
