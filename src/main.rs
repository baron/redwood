mod cli;
mod command;
mod conf;
mod context;
mod error;
mod git;
mod tmux;

use crate::cli::Cli;
use crate::command::Command;
use crate::context::Context;
use crate::error::RedwoodError;

use std::process::exit;

use clap::Parser;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Result<T> = std::result::Result<T, RedwoodError>;

fn main() {
    let cli = Cli::parse();

    let config_path = match conf::get_config_path() {
        Ok(config_path) => config_path,
        Err(e) => {
            print!("{}", e);
            exit(1);
        }
    };
    let cfg = match conf::read_config(&config_path) {
        Ok(cfg) => cfg,
        Err(RedwoodError::ConfigNotFound) => conf::Config::new(),
        Err(e) => {
            print!("{}", e);
            exit(1);
        }
    };
    let tmux = tmux::new();
    let git = git::new();
    let config_writer = conf::new_writer(&config_path);
    let ctx = Context::new(tmux, git, config_writer);

    let cmd: Box<dyn Command> = cli.into();
    if let Err(e) = cmd.execute(&ctx, cfg) {
        print!("{}", e);
        exit(1);
    }
}
