use std::path::PathBuf;

use clap::Parser;

use crate::{config::get_config_dir, logging::get_data_dir};

/// A template engine for LaTeX projects
///
/// This tool allows you to create a new LaTeX project from a
/// template. Using Templatex, you can easily scaffold LaTeX projects
/// from you templates, with the ability to set boilerplate values.
///
/// Developed by Jayan Sunil <https://github.com/jayanaxhf/templatex>
#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// The name of the project. Use `.` for the current directory.
    #[clap(required = true)]
    pub name: String,
    #[clap(flatten)]
    pub args: Args,
}

#[derive(clap::Args, Debug)]
pub struct Args {
    /// The directory containing the templates to use. This overrides
    /// the source directories specified in the config file.
    #[clap(short, long)]
    pub template_dir: Option<PathBuf>,
    /// The directory to output the project to. Defaults to the name
    /// of the project.
    #[clap(short, long)]
    pub out_dir: Option<PathBuf>,
    /// Silence all output except errors.
    #[clap(short, long)]
    pub silent: bool,
    /// Log debug output.
    #[clap(short, long)]
    pub verbose: bool,
    #[clap(long)]
    /// Log trace-level output too.
    pub very_verbose: bool,
    /// The directory to read the config from. Defaults to
    /// the standard config directory based on the OS.
    ///
    /// $XDG_CONFIG_HOME/templatex/config on Linux
    /// $HOME/Library/Application Support/templatex/config on MacOS
    /// %APPDATA%/templatex/config on Windows
    #[clap(long)]
    pub config_dir: Option<PathBuf>,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let data_dir_path = get_data_dir().display().to_string();
    let config_dir_path = get_config_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Data directory: {data_dir_path}
Config directory: {config_dir_path}
        "
    )
}
