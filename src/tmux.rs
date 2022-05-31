use tmux_interface::{AttachSession, KillSession, NewSession, SwitchClient};

use std::env;

use crate::error::RedwoodError;
use crate::Result;

pub fn new_session(session_name: &str, start_directory: &str) -> Result<()> {
    if let Err(e) = NewSession::new()
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
