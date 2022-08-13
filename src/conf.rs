use serde::{Deserialize, Serialize};
use std::env;
use std::path;

use crate::error::RedwoodError::*;
use crate::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    worktrees: Vec<WorktreeConfig>,
}

impl Config {
    pub fn add_worktree(&mut self, wt: WorktreeConfig) -> Result<()> {
        if let Some(_) = self
            .worktrees
            .iter()
            .find(|&wt2| wt2.repo_path == wt.repo_path && wt2.worktree_name == wt.worktree_name)
        {
            return Err(WorkTreeConfigAlreadyExists);
        }
        self.worktrees.push(wt);
        return Ok(());
    }

    pub fn remove_worktree(&mut self, worktree_name: &str) -> Result<()> {
        let (cfg_index, _) = match self.find(worktree_name) {
            Some(index) => index,
            None => {
                return Err(WorkTreeConfigNotFound {
                    worktree_name: String::from(worktree_name),
                })
            }
        };
        self.worktrees.remove(cfg_index);
        return Ok(());
    }

    pub fn new() -> Self {
        Config { worktrees: vec![] }
    }

    pub fn write(&self) -> Result<()> {
        let contents = match serde_json::to_string_pretty(&self) {
            Ok(contents) => contents,
            Err(msg) => return Err(ConfigWriteError(msg.to_string())),
        };

        // Make sure that the directory exists before writing to it
        let config_dir = get_config_dir()?;
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            return Err(ConfigWriteError(e.to_string()));
        }

        let config_path = get_config_path()?;

        return match std::fs::write(config_path, contents) {
            Ok(()) => Ok(()),
            Err(err) => Err(ConfigWriteError(err.to_string())),
        };
    }

    pub fn worktrees(&self) -> &Vec<WorktreeConfig> {
        return &self.worktrees;
    }

    pub fn find(&self, identifier: &str) -> Option<(usize, &WorktreeConfig)> {
        self.worktrees
            .iter()
            .enumerate()
            .find(|(_, wt)| identifier == wt.repo_path() || identifier == wt.worktree_name())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeConfig {
    repo_path: String,
    worktree_name: String,
}

impl WorktreeConfig {
    pub fn new(repo_path: &str, worktree_name: &str) -> Self {
        WorktreeConfig {
            repo_path: String::from(repo_path),
            worktree_name: String::from(worktree_name),
        }
    }

    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    pub fn worktree_name(&self) -> &str {
        &self.worktree_name
    }
}

pub fn read_config() -> Result<Config> {
    let conf_path = get_config_path()?;
    let content = match std::fs::read_to_string(conf_path) {
        Ok(content) => content,
        Err(e) => {
            return match e.kind() {
                std::io::ErrorKind::NotFound => Err(ConfigNotFound),
                _ => Err(ConfigReadError(e.to_string())),
            }
        }
    };

    let config: Config = match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(msg) => panic!("deserialize config {}", msg),
    };

    return Ok(config);
}

fn get_config_dir() -> Result<path::PathBuf> {
    return if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        Ok(path::PathBuf::from(path))
    } else if let Some(path) = env::var_os("HOME") {
        Ok(path::PathBuf::from(path).join(".config"))
    } else {
        Err(ConfigPathUnresolvable)
    };
}

fn get_config_path() -> Result<path::PathBuf> {
    let config_path = get_config_dir()?;
    return Ok(config_path.join("redwood.json"));
}
