use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, OnceLock};
use std::{env, fs};

use color_eyre::Result;
use directories::ProjectDirs;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::format::{DefaultFields, Format};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*, reload};

pub static PROJECT_NAME: LazyLock<String> =
    LazyLock::new(|| env!("CARGO_CRATE_NAME").to_uppercase().to_string());
pub static DATA_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_DATA", &*PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});

pub static LOG_ENV: LazyLock<String> = LazyLock::new(|| format!("{}_LOG_LEVEL", &*PROJECT_NAME));
pub static LOG_FILE: LazyLock<String> = LazyLock::new(|| format!("{}.log", env!("CARGO_PKG_NAME")));

pub(crate) fn get_data_dir() -> PathBuf {
    if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    }
}

pub(crate) fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "jayanaxhf", env!("CARGO_PKG_NAME"))
}

/// This handle allows enabling/disabling stdout logs
static STDOUT_FILTER_HANDLE: OnceLock<
    Arc<Handle<EnvFilter, Layered<Layer<Registry, DefaultFields, Format, File>, Registry>>>,
> = OnceLock::new();

/// INITIALIZATION -------------------------------------------------------------
pub fn init(level: LevelFilter) -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(&directory)?;

    let log_path = directory.join(&*LOG_FILE);
    let log_file = File::create(log_path)?;

    //
    // FILE LAYER
    //
    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(true);

    //
    // STDOUT LAYER
    //
    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_line_number(true);

    //
    // Reloadable filter for the stdout layer
    //
    let (stdout_filter, filter_handle) = reload::Layer::new(EnvFilter::new(level.to_string()));
    STDOUT_FILTER_HANDLE
        .set(Arc::new(filter_handle))
        .expect("Failed to set stdout filter handle");

    //
    // Store handle with erased type
    //
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer.with_filter(stdout_filter))
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}

/// RUNTIME CONTROL -------------------------------------------------------------
pub fn disable_stdout_logs() -> Result<()> {
    if let Some(handle) = STDOUT_FILTER_HANDLE.get() {
        handle.modify(|f| *f = EnvFilter::new(LevelFilter::OFF.to_string()))?;
    }
    Ok(())
}

pub fn enable_stdout_logs(level: LevelFilter) -> Result<()> {
    if let Some(handle) = STDOUT_FILTER_HANDLE.get() {
        handle.modify(|f| *f = EnvFilter::new(level.to_string()))?;
    }
    Ok(())
}
