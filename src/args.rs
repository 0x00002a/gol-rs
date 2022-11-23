use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(help = "pgm file to read initial state from")]
    pub input: PathBuf,

    #[arg(short, long, help = "threads to use")]
    pub threads: Option<u16>,

    #[arg(
        long = "bg",
        default_value_t = ' ',
        help = "character to use for completely dead regions"
    )]
    pub background: char,
}
