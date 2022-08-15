use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RedwoodError {
    ConfigWriteError(String),
    ConfigReadError(String),
    ConfigNotFound,
    ConfigPathUnresolvable,
    WorkTreeConfigNotFound { worktree_name: String },
    WorkTreeConfigAlreadyExists,
    GitError { message: String },
    CommandError { command: String, message: String },
    TmuxError(String),
    InvalidPathError { worktree_path: PathBuf, msg: String },
}

impl fmt::Display for RedwoodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use super::RedwoodError::*;
        match self {
            ConfigWriteError(msg) => {
                write!(f, "failed to write config file: {}", msg)
            }
            ConfigReadError(msg) => {
                write!(f, "failed to read config file: {}", msg)
            }
            ConfigNotFound => {
                write!(f, "config file not found")
            }
            WorkTreeConfigAlreadyExists => {
                write!(f, "work tree configuration already exists")
            }
            GitError { message } => {
                write!(f, "git failed: {}", message)
            }
            TmuxError(msg) => {
                write!(f, "{}", msg)
            }
            WorkTreeConfigNotFound { worktree_name } => {
                write!(f, "work tree configuration {} not found", worktree_name)
            }
            CommandError { command, message } => {
                write!(f, "failed to execute command \"{}\": {}", command, message)
            }
            ConfigPathUnresolvable => {
                write!(f, "could not resolve path to config variable (make sure $XDG_CONFIG_HOME or $HOME is set)")
            }
            InvalidPathError { worktree_path, msg } => {
                write!(f, "invalid path {:?}: {}", worktree_path, msg)
            }
        }
    }
}
