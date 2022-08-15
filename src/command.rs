use crate::conf::{Config, WorktreeConfig};
use crate::context::Context;
use crate::error::RedwoodError;
use crate::Result;
use crate::{cli, cli::Cli};

use std::path::{Path, PathBuf};

pub trait Command {
    fn execute(&self, ctx: &Context, cfg: Config) -> Result<()>;
}

impl std::convert::From<Cli> for Box<dyn Command> {
    // Cli::parse() must be called before this.
    fn from(cli: Cli) -> Box<dyn Command> {
        match cli.command {
            cli::Commands::New {
                repo_path,
                worktree_name,
            } => Box::new(New {
                repo_path,
                worktree_name,
            }),
            cli::Commands::Open { identifier } => Box::new(Open { identifier }),
            cli::Commands::Delete { identifier } => Box::new(Delete { identifier }),
            cli::Commands::Import { worktree_path } => Box::new(Import { worktree_path }),
            cli::Commands::List {} => Box::new(List {}),
            cli::Commands::Version {} => Box::new(Version {}),
        }
    }
}

struct New {
    repo_path: Option<PathBuf>,
    worktree_name: String,
}

impl Command for New {
    fn execute(&self, ctx: &Context, mut cfg: Config) -> Result<()> {
        let Context {
            tmux,
            git,
            config_writer,
        } = ctx;
        let default_repo_path = PathBuf::from(".");
        let repo_path = self.repo_path.as_ref().unwrap_or(&default_repo_path);
        let repo = git.get_repo_meta(repo_path)?;
        let worktree_path = repo.root_path().join(&self.worktree_name);

        cfg.add_worktree(WorktreeConfig::new(&worktree_path, &self.worktree_name))?;

        config_writer.write(&cfg)?;

        if let Err(err) = git.create_worktree(repo.root_path(), &self.worktree_name) {
            return Err(RedwoodError::from(err));
        }

        tmux.new_session(&self.worktree_name, &worktree_path)?;

        Ok(())
    }
}

struct Open {
    identifier: String,
}

impl Command for Open {
    fn execute(&self, ctx: &Context, cfg: Config) -> Result<()> {
        let Context { tmux, .. } = ctx;
        let (_, worktree_cfg) = match cfg.find(&self.identifier) {
            Some(cfg) => cfg,
            None => {
                return Err(RedwoodError::WorkTreeConfigNotFound {
                    worktree_name: self.identifier.to_string(),
                })
            }
        };

        let session_name = worktree_cfg.worktree_name();
        tmux.new_session(session_name, Path::new(worktree_cfg.repo_path()))?;
        tmux.attach_to_session(session_name)?;

        Ok(())
    }
}

struct Delete {
    identifier: String,
}

impl Command for Delete {
    fn execute(&self, ctx: &Context, mut cfg: Config) -> Result<()> {
        let Context {
            tmux,
            git,
            config_writer,
        } = ctx;
        let (_, worktree_cfg) = match cfg.find(&self.identifier) {
            Some(cfg) => cfg,
            None => {
                return Err(RedwoodError::WorkTreeConfigNotFound {
                    worktree_name: self.identifier.to_string(),
                })
            }
        };

        let repo = git.get_repo_meta(Path::new(worktree_cfg.repo_path()))?;
        if repo.is_bare() {
            git.delete_worktree(&repo.root_path(), &worktree_cfg.worktree_name())?;
        }

        tmux.kill_session(&worktree_cfg.worktree_name())?;

        cfg.remove_worktree(&self.identifier)?;
        config_writer.write(&cfg)?;

        Ok(())
    }
}

struct List {}

impl Command for List {
    fn execute(&self, _: &Context, cfg: Config) -> Result<()> {
        for worktree in cfg.list().iter() {
            println!("{}", worktree.repo_path());
        }
        Ok(())
    }
}
struct Version {}

impl Command for Version {
    fn execute(&self, _: &Context, _: Config) -> Result<()> {
        println!("{} v{}", crate::PKG_NAME, crate::PKG_VERSION);
        Ok(())
    }
}

struct Import {
    worktree_path: PathBuf,
}

impl Command for Import {
    fn execute(&self, ctx: &Context, mut cfg: Config) -> Result<()> {
        let Context { config_writer, .. } = ctx;
        let path = match self.worktree_path.canonicalize() {
            Ok(path) => path,
            Err(err) => {
                return Err(RedwoodError::InvalidPathError {
                    worktree_path: self.worktree_path.to_path_buf(),
                    msg: err.to_string(),
                })
            }
        };

        let worktree_name = path.iter().last().unwrap().to_str().unwrap();

        let wt_cfg = WorktreeConfig::new(&path, worktree_name);
        cfg.add_worktree(wt_cfg)?;
        config_writer.write(&cfg)?;

        Ok(())
    }
}
