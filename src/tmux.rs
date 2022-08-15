use crate::error::RedwoodError;
use crate::Result;

use tmux_interface::{AttachSession, KillSession, SwitchClient, TmuxCommand};

use std::env;
use std::path::{Path, PathBuf};

pub trait Tmux {
    fn new_session(&self, session_name: &str, start_directory: &Path) -> Result<()>;
    fn kill_session(&self, session_name: &str) -> Result<()>;
    fn attach_to_session(&self, session_name: &str) -> Result<()>;
}

pub fn new() -> impl Tmux {
    TmuxCommand::new()
}

impl Tmux for TmuxCommand<'_> {
    fn new_session(&self, session_name: &str, start_directory: &Path) -> Result<()> {
        let tmux_config_path = get_tmux_config_path()?;
        if let Err(e) = TmuxCommand::new()
            .file(tmux_config_path.to_string_lossy())
            .new_session()
            .detached()
            .session_name(session_name)
            .start_directory(start_directory.to_string_lossy())
            .output()
        {
            return Err(RedwoodError::from(e));
        }
        return Ok(());
    }

    fn kill_session(&self, session_name: &str) -> Result<()> {
        return match KillSession::new().target_session(session_name).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(RedwoodError::from(e)),
        };
    }

    fn attach_to_session(&self, session_name: &str) -> Result<()> {
        let res = if in_tmux_session() {
            SwitchClient::new().target_session(session_name).output()
        } else {
            AttachSession::new().target_session(session_name).output()
        };

        return match res {
            Ok(_) => Ok(()),
            Err(e) => Err(RedwoodError::from(e)),
        };
    }
}

fn in_tmux_session() -> bool {
    env::var_os("TMUX").is_some()
}

fn get_tmux_config_path() -> Result<PathBuf> {
    let configs_dir_path = if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os("HOME") {
        PathBuf::from(path).join(".config")
    } else {
        return Err(RedwoodError::ConfigPathUnresolvable);
    };
    return Ok(configs_dir_path.join("tmux").join(".tmux.conf"));
}

impl From<tmux_interface::Error> for RedwoodError {
    fn from(error: tmux_interface::Error) -> Self {
        RedwoodError::TmuxError(error.to_string())
    }
}
