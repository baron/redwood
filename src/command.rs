use crate::context::Context;
use crate::error::RedwoodError;
use crate::Result;
use crate::{cli, cli::Cli};

use log::debug;

use std::path::PathBuf;
use std::{env, fs};
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use serde_json;

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

    let entries: Vec<_> = dir.collect::<Vec<_>>().into_iter().map(|entry| entry.map_err(|err| RedwoodError::FSError {
        path: path.to_path_buf(),
        msg: err.to_string(),
    })).collect::<std::result::Result<Vec<_>, RedwoodError>>()?;

    let subdirs: Vec<_> = entries
        .into_par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                collect_dirs(&path, ignored).ok()
            } else {
                None
            }
        })
        .flatten()
        .collect();

    result.extend(subdirs);
    Ok(result)
}

fn get_cache_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
    PathBuf::from(home_dir).join(".redwood_cache.json")
}

fn load_cache() -> io::Result<Vec<PathBuf>> {
    let cache_path = get_cache_path();
    if !cache_path.exists() {
        return Ok(vec![]);
    }
    let mut file = File::open(cache_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let paths: Vec<String> = serde_json::from_str(&contents)?;
    Ok(paths.into_iter().map(PathBuf::from).collect())
}

fn save_cache(paths: &[PathBuf]) -> io::Result<()> {
    let cache_path = get_cache_path();
    let paths: Vec<String> = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let contents = serde_json::to_string(&paths)?;
    let mut file = File::create(cache_path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
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

        debug!("redwood list: beginning scan under {:?}", home_path);

        let ignored_dirs_var = env::var("REDWOOD_IGNORED_DIRS").unwrap_or(String::from(""));
        let mut ignored_dirs = ignored_dirs_var
            .split(',')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        // bare repositories maintain a "worktrees"-directory in their root containing git info for
        // each worktree. If we don't ignore these then we'll get duplicate entries for each
        // worktree, e.g. <repo>/worktrees/my-worktree and <repo>/my-worktree.
        ignored_dirs.push("worktrees");

        // Load the cache
        let cached_dirs = load_cache().unwrap_or_else(|_| vec![]);
        debug!("redwood list: loaded {} repositories from cache", cached_dirs.len());

        // Scan the file system for new repositories
        let mut dirs = collect_dirs(&home_path, &ignored_dirs)?;

        dirs = dirs
            .into_iter()
            .filter_map(|d| {
                match git2::Repository::open(&d) {
                    Ok(repo) => {
                        // Apply optional filters
                        if (self.only_bare_repos && !repo.is_bare())
                            || (self.only_worktrees && !repo.is_worktree())
                        {
                            None
                        } else {
                            Some(d)
                        }
                    }
                    Err(_) => None, // not a repo
                }
            })
            .collect();

        debug!("redwood list: found {} repositories after filtering", dirs.len());

        // Merge cached and real-time data
        let mut merged_dirs: Vec<PathBuf> = cached_dirs.into_iter().filter(|d| d.exists()).collect();
        for d in dirs {
            if !merged_dirs.contains(&d) {
                merged_dirs.push(d);
            }
        }

        debug!("redwood list: merged {} repositories", merged_dirs.len());

        // Save the merged results to the cache
        if let Err(e) = save_cache(&merged_dirs) {
            eprintln!("Failed to save cache: {}", e);
        }

        merged_dirs.iter().for_each(|d| println!("{}", d.display()));

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
