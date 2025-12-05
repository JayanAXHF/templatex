use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[clap(required = true)]
    pub name: String,
    #[clap(flatten)]
    pub args: Args,
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(short, long)]
    pub template_dir: Option<PathBuf>,
    #[clap(short, long)]
    pub out_dir: Option<PathBuf>,
    #[clap(short, long)]
    pub silent: bool,
    #[clap(short, long)]
    pub verbose: bool,
    #[clap(long)]
    pub very_verbose: bool,
}
