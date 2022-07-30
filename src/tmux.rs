use tmux_interface::{AttachSession, KillSession, SwitchClient, TmuxCommand};

use std::env;
use std::path;

use crate::error::RedwoodError;
use crate::Result;

pub fn new_session(session_name: &str, start_directory: &str) -> Result<()> {
    let tmux_config_path = get_tmux_config_path()?;
    if let Err(e) = TmuxCommand::new()
        .file(tmux_config_path.to_string_lossy())
        .new_session()
        .detached()
        .session_name(session_name)
        .start_directory(start_directory)
        .output()
    {
        return Err(RedwoodError::TmuxError(e.to_string()));
    }
    return Ok(());
}

pub fn new_session_attached(session_name: &str, start_directory: &str) -> Result<()> {
    new_session(session_name, start_directory)?;
    if in_tmux_session() {
        // Directly attaching to the session does not seem to work when creating it from inside another
        // tmux session, so create it detached and then switch to it instead.
        switch_session(session_name)?;
    } else {
        attach_session(session_name)?;
    }
    Ok(())
}

fn switch_session(session_name: &str) -> Result<()> {
    return match SwitchClient::new().target_session(session_name).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(RedwoodError::TmuxError(e.to_string())),
    };
}

fn attach_session(session_name: &str) -> Result<()> {
    return match AttachSession::new().target_session(session_name).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(RedwoodError::TmuxError(e.to_string())),
    };
}

pub fn kill_session(session_name: &str) -> Result<()> {
    return match KillSession::new().target_session(session_name).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(RedwoodError::TmuxError(e.to_string())),
    };
}

fn in_tmux_session() -> bool {
    env::var_os("TMUX").is_some()
}

fn get_tmux_config_path() -> Result<path::PathBuf> {
    let configs_dir_path = if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        path::PathBuf::from(path)
    } else if let Some(path) = env::var_os("HOME") {
        path::PathBuf::from(path).join(".config")
    } else {
        return Err(RedwoodError::ConfigPathUnresolvable);
    };
    return Ok(configs_dir_path.join("tmux").join(".tmux.conf"));
}
