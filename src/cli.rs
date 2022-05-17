use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "redwood")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(arg_required_else_help = true)]
    New {
        #[clap(required = true)]
        worktree_name: String,
        #[clap(required = false)]
        repo_path: Option<String>,
    },
    Open {
        #[clap(required = true)]
        worktree_name: String,
    },
    Delete {
        #[clap(required = true)]
        worktree_name: String,
    },
    List {},
    Version {},
}
