#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// the command line interface structure
#[derive(Parser, Debug)]
#[command(name = "layerscript", version = "0.1.0")]
pub struct Cli {
    /// show more logs
    #[arg(short, long, global = true)]
    pub Verbose: bool,

    /// specify the workspace folder
    #[arg(short, long, global = true, value_name = "DIR")]
    pub Workspace: Option<PathBuf>,

    /// the command to actually run
    #[command(subcommand)]
    pub Command: Commands,
}

/// the different things layerscript can do
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// turn layerscript code into binary
    Compile {
        /// the file to read
        #[arg(required = true, value_name = "FILE")]
        Input: PathBuf,

        /// optimization level (0-3)
        #[arg(short = 'O', long, default_value_t = 2)]
        OptLevel: u8,
    },

    /// run a tiny script in the repl
    Eval {
        /// the layerscript code string
        #[arg(required = true)]
        Code: String,
    },
    /// run all the workspace tests
    Test {
        /// only run tests that match this
        #[arg(short, long)]
        Filter: Option<String>,
    },
}

impl Cli {
    /// helper to parse args from the env
    pub fn ParseArgs() -> Self {
        Self::parse()
    }

    /// helper for testing the cli without a terminal
    pub fn ParseFromArgs<I, T>(Args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(Args)
    }
}
