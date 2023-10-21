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
        #[clap(required = true, parse(from_os_str))]
        repo_path: PathBuf,
        #[clap(required = true)]
        worktree_name: String,
        #[clap(long)]
        tmux_session_name: Option<String>,
    },
    /// Open existing worktree configuration
    Open {
        #[clap(required = true)]
        path: PathBuf,
    },
    /// Delete worktree configuration
    Delete {
        #[clap(required = true)]
        path: PathBuf,
    },
    /// List existing worktree configurations
    List {
        /// Only list bare repositories
        #[clap(long)]
        only_bare_repos: bool,
        /// Only list work trees (in bare repositories).
        #[clap(long)]
        only_worktrees: bool,
    },
    /// Print version of Redwood
    Version {},
}
