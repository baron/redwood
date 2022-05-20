use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "redwood")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create new worktree
    #[clap(arg_required_else_help = true)]
    New {
        #[clap(required = true)]
        worktree_name: String,
        #[clap(required = false, parse(from_os_str))]
        repo_path: Option<PathBuf>,
    },
    /// Open existing worktree configuration
    Open {
        #[clap(required = true)]
        worktree_name: String,
    },
    /// Delete worktree configuration
    Delete {
        #[clap(required = true)]
        worktree_name: String,
    },
    /// Import existing worktree
    Import {
        #[clap(required = true, parse(from_os_str))]
        worktree_path: PathBuf,
    },
    /// List existing worktree configurations
    List {},
    /// Print version of Redwood
    Version {},
}
