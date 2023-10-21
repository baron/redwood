use crate::context::Context;
use crate::error::RedwoodError;
use crate::Result;
use crate::{cli, cli::Cli};

use std::path::PathBuf;
use std::{env, fs};

pub trait Command {
    fn execute(&self, ctx: &Context) -> Result<()>;
}

impl std::convert::From<Cli> for Box<dyn Command> {
    // Cli::parse() must be called before this.
    fn from(cli: Cli) -> Box<dyn Command> {
        match cli.command {
            cli::Commands::New {
                repo_path,
                worktree_name,
                tmux_session_name,
            } => Box::new(New {
                repo_path,
                worktree_name,
                tmux_session_name,
            }),
            cli::Commands::Open { path } => Box::new(Open { path }),
            cli::Commands::Delete { path } => Box::new(Delete { path }),
            cli::Commands::List {
                only_bare_repos,
                only_worktrees,
            } => Box::new(List {
                only_bare_repos,
                only_worktrees,
            }),
            cli::Commands::Version {} => Box::new(Version {}),
        }
    }
}

struct New {
    repo_path: PathBuf,
    worktree_name: String,
    tmux_session_name: Option<String>,
}

impl Command for New {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let Context { tmux, git } = ctx;
        let repo = git.get_repo_meta(&self.repo_path)?;
        if !repo.is_bare() {
            return Err(RedwoodError::NotBareRepoError {
                repo_path: self.repo_path.to_path_buf(),
            });
        }
        let worktree_path = repo.root_path().join(&self.worktree_name);

        git.create_worktree(repo.root_path(), &self.worktree_name)?;

        let session_name = self
            .tmux_session_name
            .as_deref()
            .unwrap_or(&self.worktree_name);

        tmux.new_session(session_name, &worktree_path)?;
        tmux.attach_to_session(session_name)?;

        Ok(())
    }
}

struct Open {
    path: PathBuf,
}

impl Command for Open {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let Context { tmux, .. } = ctx;

        let normalized = match self.path.canonicalize() {
            Ok(path) => path,
            Err(err) => {
                return Err(RedwoodError::PathError {
                    path: self.path.to_path_buf(),
                    msg: format!("failed to canonicalize: {}", err),
                })
            }
        };
        let session_name = normalized.iter().last().unwrap().to_str().unwrap();

        tmux.new_session(session_name, &self.path)?;
        tmux.attach_to_session(session_name)?;

        Ok(())
    }
}

struct Delete {
    path: PathBuf,
}

impl Command for Delete {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let Context { tmux: _, git } = ctx;

        let repo = git.get_repo_meta(&self.path)?;

        if !repo.is_bare() {
            return Err(RedwoodError::NotBareRepoError {
                repo_path: self.path.to_path_buf(),
            });
        }
        git.delete_worktree(&self.path)?;

        Ok(())
    }
}

struct List {
    only_bare_repos: bool,
    only_worktrees: bool,
}

fn collect_dirs(path: &std::path::Path, ignored: &[&str]) -> Result<Vec<PathBuf>> {
    let is_hidden = path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .starts_with('.');
    if !path.is_dir() || path.is_symlink() || is_hidden {
        return Ok(vec![]);
    }

    let dir = match fs::read_dir(path) {
        Ok(dir) => dir,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                return Ok(vec![]);
            }
            return Err(RedwoodError::FSError {
                path: path.to_path_buf(),
                msg: err.to_string(),
            });
        }
    };

    if ignored.iter().any(|ignored_dir| {
        path.iter()
            .any(|p| p.to_str().unwrap_or("").starts_with(ignored_dir))
    }) {
        return Ok(vec![]);
    }

    let mut result: Vec<PathBuf> = vec![path.to_path_buf()];

    for entry in dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    continue;
                }
                return Err(RedwoodError::FSError {
                    path: path.to_path_buf(),
                    msg: err.to_string(),
                });
            }
        };

        let path = entry.path();
        if path.is_dir() {
            let res = collect_dirs(&path, ignored)?;
            result.extend(res);
        }
    }
    Ok(result)
}

impl Command for List {
    fn execute(&self, _: &Context) -> Result<()> {
        let home_dir = match env::var("HOME") {
            Ok(home_dir) => home_dir,
            Err(err) => {
                return Err(RedwoodError::EnvironmentVariableError {
                    var: String::from("HOME"),
                    msg: err.to_string(),
                })
            }
        };
        let home_path = PathBuf::from(home_dir);

        let ignored_dirs_var = env::var("REDWOOD_IGNORED_DIRS").unwrap_or(String::from(""));
        let mut ignored_dirs = ignored_dirs_var
            .split(',')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        // bare repositories maintain a "worktrees"-directory in their root containing git info for
        // each worktree. If we don't ignore these then we'll get duplicate entries for each
        // worktree, e.g. <repo>/worktrees/my-worktree and <repo>/my-worktree.
        ignored_dirs.push("worktrees");

        let mut dirs = collect_dirs(&home_path, &ignored_dirs)?;

        dirs = dirs
            .into_iter()
            // we only care about directories that are git repositories
            .filter(|d| git2::Repository::open(d).is_ok())
            .filter(|d| {
                if self.only_bare_repos {
                    git2::Repository::open(d).is_ok_and(|repo| repo.is_bare())
                } else {
                    true
                }
            })
            .filter(|d| {
                if self.only_worktrees {
                    git2::Repository::open(d).is_ok_and(|repo| repo.is_worktree())
                } else {
                    true
                }
            })
            .collect();

        dirs.iter().for_each(|d| println!("{}", d.display()));

        Ok(())
    }
}

struct Version {}

impl Command for Version {
    fn execute(&self, _: &Context) -> Result<()> {
        println!("{} v{}", crate::PKG_NAME, crate::PKG_VERSION);
        Ok(())
    }
}
