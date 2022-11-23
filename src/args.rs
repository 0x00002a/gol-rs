use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    pub input: PathBuf,

    #[arg(short, long)]
    pub threads: Option<u16>,

    #[arg(short, long, default_value_t = ' ')]
    pub background: char,
}
