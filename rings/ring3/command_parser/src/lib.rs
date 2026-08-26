#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "burst", version = "0.1.0")]
pub struct cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true, value_name = "DIR")]
    pub workspace: Option<PathBuf>,

    #[command(subcommand)]
    pub command: commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum commands {
    // compile file
    compile {
        #[arg(required = true, value_name = "FILE")]
        input: PathBuf,

        #[arg(short = 'O', long, default_value_t = 2)]
        opt_level: u8,
    },

    // run inline statement
    eval {
        #[arg(required = true)]
        code: String,
    },
    // run tests
    test {
        #[arg(short, long)]
        filter: Option<String>,
    },
}

impl cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    // helper for tests/repl
    pub fn parse_from_args<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(args)
    }
}
