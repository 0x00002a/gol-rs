use clap::{builder::PossibleValue, Parser, ValueEnum};
use std::path::PathBuf;

use crate::bgrid::Charset;

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

    #[arg(value_enum, long, default_value_t = Charset::Braille, help = "unicode character set to use for rendering")]
    pub charset: Charset,
}

impl ValueEnum for Charset {
    fn value_variants<'a>() -> &'a [Self] {
        &[Charset::Block, Charset::Braille]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Charset::Braille => PossibleValue::new("braille"),
            Charset::Block => PossibleValue::new("block"),
        })
    }
}
