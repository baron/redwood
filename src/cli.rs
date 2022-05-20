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
        #[clap(required = false)]
        repo_path: Option<String>,
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
        #[clap(required = true)]
        worktree_path: String,
    },
    /// List existing worktree configurations
    List {},
    /// Print version of Redwood
    Version {},
}
