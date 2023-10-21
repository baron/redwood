use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum RedwoodError {
    ConfigPathUnresolvable,
    GitError { message: String },
    CommandError { command: String, message: String },
    TmuxError(String),
    PathError { path: PathBuf, msg: String },
    NotBareRepoError { repo_path: PathBuf },
    EnvironmentVariableError { var: String, msg: String },
    FSError { path: PathBuf, msg: String },
}

impl fmt::Display for RedwoodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use super::RedwoodError::*;
        match self {
            GitError { message } => {
                write!(f, "git failed: {}", message)
            }
            TmuxError(msg) => {
                write!(f, "{}", msg)
            }
            CommandError { command, message } => {
                write!(f, "failed to execute command \"{}\": {}", command, message)
            }
            ConfigPathUnresolvable => {
                write!(f, "could not resolve path to config variable (make sure $XDG_CONFIG_HOME or $HOME is set)")
            }
            PathError { path, msg } => {
                write!(f, "path error {:?}: {}", path, msg)
            }
            NotBareRepoError { repo_path } => {
                write!(f, "repo at {:?} is not bare", repo_path)
            }
            EnvironmentVariableError { var, msg } => {
                write!(f, "could not get environment variable {}: {}", var, msg)
            }
            FSError { path, msg } => {
                write!(f, "could not access path {:?}: {}", path, msg)
            }
        }
    }
}
