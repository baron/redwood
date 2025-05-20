mod cli;
mod command;
mod context;
mod error;
mod git;
mod tmux;
mod user;

use crate::cli::Cli;
use crate::command::Command;
use crate::context::Context;
use crate::error::RedwoodError;

use std::process::exit;

use clap::Parser;
use env_logger;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Result<T> = std::result::Result<T, RedwoodError>;

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let tmux = tmux::new();
    let git = git::new();
    let ctx = Context::new(tmux, git);

    let cmd: Box<dyn Command> = cli.into();
    if let Err(e) = cmd.execute(&ctx) {
        print!("{}", e);
        exit(1);
    }
}
