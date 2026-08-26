#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "burst", version = "0.1.0")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub Verbose: bool,

    #[arg(short, long, global = true, value_name = "DIR")]
    pub Workspace: Option<PathBuf>,

    #[command(subcommand)]
    pub Command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    // compile file
    Compile {
        #[arg(required = true, value_name = "FILE")]
        Input: PathBuf,

        #[arg(short = 'O', long, default_value_t = 2)]
        OptLevel: u8,
    },

    // run inline statement
    Eval {
        #[arg(required = true)]
        Code: String,
    },
    // run tests
    Test {
        #[arg(short, long)]
        Filter: Option<String>,
    },
}

impl Cli {
    pub fn ParseArgs() -> Self {
        Self::parse()
    }

    // helper for tests/repl
    pub fn ParseFromArgs<I, T>(Args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(Args)
    }
}
