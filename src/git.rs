use crate::error::RedwoodError;
use crate::Result;

use std::path::{Path, PathBuf};

use git2::{Repository as git2Repository, WorktreePruneOptions};

pub trait Git {
    fn create_worktree(&self, repo_path: &Path, worktree_name: &str) -> Result<()>;
    fn delete_worktree(&self, worktree_path: &Path) -> Result<()>;
    fn get_repo_meta(&self, repo_path: &Path) -> Result<RepoMeta>;
}

pub struct RepoMeta {
    is_bare: bool,
    root_path: PathBuf,
}

impl RepoMeta {
    pub fn is_bare(&self) -> bool {
        self.is_bare
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }
}

struct GitImpl {}

pub fn new() -> impl Git {
    GitImpl {}
}

impl GitImpl {
    fn open_repo(path: &Path) -> Result<git2Repository> {
        match git2Repository::open(path) {
            Ok(repo) => Ok(repo),
            Err(err) => Err(RedwoodError::from(err)),
        }
    }

    fn get_repo_root(repo_path: &Path) -> Result<PathBuf> {
        let repo = GitImpl::open_repo(repo_path)?;
        let mut repo_root = repo.path().to_path_buf();

        // If the repository is a worktree then the repo.path() call will return
        // <repo>/worktrees/<worktree_name>, so the last two components need to be stripped.
        // TODO: There is probably a better way to do this.
        if repo.is_worktree() {
            let components = repo_root.components().collect::<Vec<_>>();
            repo_root = components
                .iter()
                .take(components.len() - 2)
                .fold(PathBuf::new(), |path, component| path.join(component));
        }
        Ok(repo_root)
    }
}

impl Git for GitImpl {
    fn create_worktree(&self, repo_path: &Path, worktree_name: &str) -> Result<()> {
        let repo = GitImpl::open_repo(repo_path)?;
        let repo_root = GitImpl::get_repo_root(repo_path)?;
        let worktree_path = repo_root.join(worktree_name);

        // When deleting a worktree the HEAD still remains in refs/heads.
        // git, when invoked directly, will account for its potential existence when creating a
        // worktree; whereas when invoked through git2 (or via libgit2) does not seem to account for it
        // and will throw an error.
        // This means that using redwood to create, then delete, and then recreate a worktree using the
        // same name would fail when using git2, so we invoke git directly instead for this.
        let mut cmd = std::process::Command::new("git");
        cmd.args([
            "worktree",
            "add",
            "-b",
            worktree_name,
            &worktree_path.to_string_lossy(),
        ]);

        let has_remote_branch =
            |name: &str| repo.find_branch(name, git2::BranchType::Remote).is_ok();
        // check for origin/main or origin/master branch and reset to it
        if repo.is_bare() {
            if has_remote_branch("origin/main") {
                cmd.arg("origin/main");
            } else if has_remote_branch("origin/master") {
                cmd.arg("origin/master");
            }
        }

        match cmd.output() {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(RedwoodError::CommandError {
                command: format!("{:?}", cmd),
                message: String::from_utf8(output.stderr).unwrap(),
            }),
            Err(e) => Err(RedwoodError::CommandError {
                command: format!("{:?}", cmd),
                message: e.to_string(),
            }),
        }
    }

    fn get_repo_meta(&self, path: &Path) -> Result<RepoMeta> {
        let repo_root = GitImpl::get_repo_root(path)?;
        let repo = GitImpl::open_repo(&repo_root)?;

        Ok(RepoMeta {
            root_path: repo_root,
            is_bare: repo.is_bare(),
        })
    }

    fn delete_worktree(&self, worktree_path: &Path) -> Result<()> {
        let repo = GitImpl::open_repo(worktree_path)?;
        let worktree_name = worktree_path.file_name().unwrap().to_str().unwrap(); // TODO: Get rid of unwraps

        let worktree = match repo.find_worktree(worktree_name) {
            Ok(worktree) => worktree,
            Err(e) => return Err(RedwoodError::from(e)),
        };

        if let Err(e) = worktree.prune(Some(
            WorktreePruneOptions::new().valid(true).working_tree(true),
        )) {
            return Err(RedwoodError::from(e));
        }

        let branch = match repo.find_branch(worktree_name, git2::BranchType::Local) {
            Ok(branch) => Some(branch),
            Err(e) => match e.code() {
                git2::ErrorCode::NotFound => None,
                _ => return Err(RedwoodError::from(e)),
            },
        };

        if let Some(mut branch) = branch {
            if let Err(e) = branch.delete() {
                return Err(RedwoodError::from(e));
            }
        }

        Ok(())
    }
}

impl From<git2::Error> for RedwoodError {
    fn from(error: git2::Error) -> Self {
        RedwoodError::GitError {
            message: error.message().to_owned(),
        }
    }
}
