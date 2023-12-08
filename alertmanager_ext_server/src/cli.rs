use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct Cli {
    /// Path to the config file in yaml format
    #[clap(
        short = 'f',
        long,
        default_value = "config.yaml",
        env = "alertmanager_ext_config_file"
    )]
    pub config_file: PathBuf,
}
