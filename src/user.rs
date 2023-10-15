use std::{env, path::PathBuf};

use crate::{error::RedwoodError, Result};

pub fn get_user_config_directory() -> Result<PathBuf> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("HOME") {
        return Ok(PathBuf::from(path).join(".config"));
    }
    Err(RedwoodError::ConfigPathUnresolvable)
}
