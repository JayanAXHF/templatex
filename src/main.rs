use std::path::PathBuf;

use clap::Parser;
use templatex::{
    cli, config,
    errors::Result,
    logging::init,
    templating::{self, LoadableDir},
    tui::picker,
};
use tracing::level_filters::LevelFilter;

fn main() -> color_eyre::Result<()> {
    let cli::Cli { name, args } = cli::Cli::parse();
    let config = config::Settings::new().unwrap();
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
        .map(PathBuf::load_dir)
        .collect::<Result<Vec<_>>>()?;

    let sel = if loaded_templates.len() == 1 {
        loaded_templates[0].clone()
    } else {
        let theme = config.get_theme();
        picker(loaded_templates, theme)?
    };
    println!("\r\n");

    let engine = templating::EngineBuilder::default()
        .template_dirs([sel.dir])
        .clone()
        .build()?;

    let template = engine.get_template(sel.name.as_str()).unwrap();
    let vars = template
        .files()
        .iter()
        .flat_map(|f| f.variables())
        .collect::<Vec<_>>();

    let data = vars
        .iter()
        .map(|v| {
            let prompt = inquire::prompt_text(format!("Enter value for {}", v)).unwrap_or_default();
            (v.to_string(), prompt)
        })
        .collect::<Vec<_>>();

    let out_dir = args.out_dir.unwrap_or_else(|| PathBuf::from(name));

    engine.render(&out_dir, sel.name.as_str(), &data)?;

    Ok(())
}
