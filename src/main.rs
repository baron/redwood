mod cli;
mod conf;
mod error;
mod git;
mod tmux;

use std::path::Path;
use std::process::exit;

use crate::cli::{Cli, Commands};
use crate::error::RedwoodError;
use clap::Parser;

pub type Result<T> = std::result::Result<T, RedwoodError>;

fn main() {
    let cfg = match conf::read_config() {
        Ok(cfg) => cfg,
        Err(RedwoodError::ConfigNotFound) => conf::Config::new(),
        Err(e) => {
            print!("{}", e);
            exit(1);
        }
    };
    let args = Cli::parse();

    if let Err(e) = match args.command {
        Commands::New {
            repo_path,
            worktree_name,
        } => new(cfg, worktree_name, repo_path),
        Commands::Open { worktree_name } => open(cfg, worktree_name),
        Commands::Delete { worktree_name } => delete(cfg, worktree_name),
        Commands::List {} => list(cfg),
    } {
        print!("{}", e);
        exit(1);
    }
}

fn new(mut cfg: conf::Config, worktree_name: String, repo_path: Option<String>) -> Result<()> {
    let repo_path = repo_path.unwrap_or(String::from("."));
    let repo = git::open_repo(Path::new(&repo_path))?;
    let repo_root = git::get_repo_root(&repo);
    let worktree_path = repo_root.join(&worktree_name);

    cfg.add_worktree(conf::WorktreeConfig::new(
        worktree_path.to_str().unwrap(),
        &worktree_name,
    ))?;

    cfg.write()?;

    if let Err(RedwoodError::GitError {
        code: git2::ErrorCode::Exists,
        class: git2::ErrorClass::Reference,
        ..
    }) = git::create_worktree(&repo_root, &worktree_name)
    {}

    return tmux::new_session(&worktree_name, worktree_path.to_str().unwrap());
}

fn list(cfg: conf::Config) -> Result<()> {
    for worktree in cfg.worktrees().iter() {
        println!("{}", worktree.repo_path());
    }
    return Ok(());
}

fn open(cfg: conf::Config, worktree_name: String) -> Result<()> {
    let worktree_cfg = match cfg
        .worktrees()
        .iter()
        .find(|wt| wt.worktree_name() == worktree_name)
    {
        Some(cfg) => cfg,
        None => return Err(RedwoodError::WorkTreeConfigNotFound { worktree_name }),
    };

    return tmux::new_session_attached(&worktree_name, worktree_cfg.repo_path());
}

fn delete(mut cfg: conf::Config, worktree_name: String) -> Result<()> {
    let worktree_cfg = match cfg
        .worktrees()
        .iter()
        .find(|wt| wt.worktree_name() == worktree_name)
    {
        Some(cfg) => cfg,
        None => return Err(RedwoodError::WorkTreeConfigNotFound { worktree_name }),
    };

    let repo = git::open_repo(Path::new(worktree_cfg.repo_path()))?;

    git::prune_worktree(&repo, &worktree_name)?;

    cfg.remove_worktree(&worktree_name)?;
    cfg.write()?;

    tmux::kill_session(&worktree_name)?;

    return Ok(());
}
