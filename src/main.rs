use std::path::PathBuf;

use clap::Parser;
use ratatui::crossterm::terminal::disable_raw_mode;
use templatex::{
    cli, config,
    filter::Filter,
    logging::{disable_stdout_logs, enable_stdout_logs, init},
    templating::{self, LoadableDir},
    tui::picker,
};
use tracing::{debug, level_filters::LevelFilter};

fn main() -> color_eyre::Result<()> {
    let cli::Cli { name, args } = cli::Cli::parse();
    let config = if let Some(cdir) = args.config_dir {
        config::Settings::with_source_dir(cdir)?
    } else {
        config::Settings::new()?
    };
    let sources = config.get_source_dirs();
    let level = if args.very_verbose {
        LevelFilter::TRACE
    } else if args.silent {
        LevelFilter::OFF
    } else if args.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    init(level)?;

    enable_stdout_logs(level)?;
    tracing::info!("Starting up");
    disable_stdout_logs()?;

    let template_dirs = match args.template_dir {
        Some(p) => vec![p],
        None => sources
            .iter()
            .flat_map(|f| {
                f.read_dir()
                    .unwrap()
                    .filter_map(|e| {
                        let e = e.ok()?;
                        if e.file_type().ok()?.is_dir() {
                            return Some(e.path());
                        }
                        None
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    };

    let loaded_templates = template_dirs
        .iter()
        .filter_map(|p| {
            let p = p.load_dir();
            let Ok(p) = p else {
                let e = p.unwrap_err();
                tracing::error!("Failed to load template dir: {}", e);
                return None;
            };
            debug!("Loaded template dir: {:?}", p);

            if p.config.ignore { None } else { Some(p) }
        })
        .collect::<Vec<_>>();

    if loaded_templates.is_empty() {
        tracing::error!("No templates found");
        return Ok(());
    }
    let sel = if loaded_templates.len() == 1 {
        loaded_templates[0].clone()
    } else {
        let theme = config.get_theme();
        picker(loaded_templates, theme)?
    };
    disable_raw_mode()?;
    println!("\r\n");
    let include_filters = sel
        .config
        .include
        .as_ref()
        .map(Filter::<String>::with_filter);

    let exclude_filters = sel
        .config
        .exclude
        .as_ref()
        .map(Filter::<String>::with_filter);

    let engine = templating::EngineBuilder::default()
        .include_filters(include_filters)
        .exclude_filters(exclude_filters)
        .template_dirs([sel.dir().clone()])
        .clone()
        .build()?;

    let t_name = &sel.dir().file_name().unwrap().display().to_string();
    let template = engine.get_template(t_name).unwrap();
    let vars = template
        .files()
        .iter()
        .flat_map(|f| f.variables())
        .collect::<Vec<_>>();

    let data = vars
        .iter()
        .map(|v| {
            let prompt = inquire::prompt_text(format!("Enter value for {}", v)).unwrap();
            (v.to_string(), prompt)
        })
        .collect::<Vec<_>>();

    let out_dir = args.out_dir.unwrap_or_else(|| PathBuf::from(name));

    engine.render(&out_dir, t_name, &data)?;

    Ok(())
}
