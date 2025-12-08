use derive_more::Display;
use std::env;
use thiserror::Error;
use tracing::error;

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

pub fn init() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let metadata = metadata!();
            let file_path = handle_dump(&metadata, panic_info);
            // prints human-panic message
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        error!("Error: {}", msg);

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(1);
    }));
    Ok(())
}
