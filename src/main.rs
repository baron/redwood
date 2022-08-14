mod cli;
mod conf;
mod error;
mod git;
mod tmux;

use std::path::{Path, PathBuf};
use std::process::exit;

use crate::cli::{Cli, Commands};
use crate::error::RedwoodError;
use clap::Parser;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        Commands::Open { identifier } => open(cfg, &identifier),
        Commands::Delete { identifier } => delete(cfg, &identifier),
        Commands::Import { worktree_path } => import(cfg, &worktree_path),
        Commands::List {} => list(cfg),
        Commands::Version {} => version(),
    } {
        print!("{}", e);
        exit(1);
    }
}

fn new(mut cfg: conf::Config, worktree_name: String, repo_path: Option<PathBuf>) -> Result<()> {
    let repo_path = repo_path.unwrap_or(PathBuf::from("."));
    let repo = git::open_repo(&repo_path)?;
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
    for worktree in cfg.list().iter() {
        println!("{}", worktree.repo_path());
    }
    return Ok(());
}

fn open(cfg: conf::Config, identifier: &str) -> Result<()> {
    let (_, worktree_cfg) = match cfg.find(identifier) {
        Some(cfg) => cfg,
        None => {
            return Err(RedwoodError::WorkTreeConfigNotFound {
                worktree_name: identifier.to_string(),
            })
        }
    };

    return tmux::new_session_attached(worktree_cfg.worktree_name(), worktree_cfg.repo_path());
}

fn delete(mut cfg: conf::Config, identifier: &str) -> Result<()> {
    let (_, worktree_cfg) = match cfg.find(identifier) {
        Some(cfg) => cfg,
        None => {
            return Err(RedwoodError::WorkTreeConfigNotFound {
                worktree_name: identifier.to_string(),
            })
        }
    };

    let repo = git::open_repo(Path::new(worktree_cfg.repo_path()))?;
    if repo.is_bare() {
        git::prune_worktree(&repo, &worktree_cfg.worktree_name())?;
    }

    tmux::kill_session(&worktree_cfg.worktree_name())?;

    cfg.remove_worktree(&identifier)?;
    cfg.write()?;

    return Ok(());
}

fn import(mut cfg: conf::Config, worktree_path: &Path) -> Result<()> {
    let path = match worktree_path.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            return Err(RedwoodError::InvalidPathError {
                worktree_path: worktree_path.to_path_buf(),
                msg: err.to_string(),
            })
        }
    };

    let repo = git::open_repo(&worktree_path)?;

    let worktree_name = path.iter().last().unwrap().to_str().unwrap();
    if repo.is_bare() {
        git::find_worktree(&repo, worktree_name)?; // ensure the worktree exists
    }

    let wt_cfg = conf::WorktreeConfig::new(path.to_str().unwrap(), worktree_name);
    cfg.add_worktree(wt_cfg)?;
    cfg.write()?;

    return Ok(());
}

fn version() -> Result<()> {
    println!("{} v{}", PKG_NAME, PKG_VERSION);
    Ok(())
}
