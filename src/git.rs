use crate::error::RedwoodError;
use crate::Result;
use git2::{Repository, WorktreePruneOptions};
use std::path::{Path, PathBuf};

pub fn create_worktree(repo_path: &Path, worktree_name: &str) -> Result<()> {
    let repo = open_repo(repo_path)?;
    let repo_root = get_repo_root(&repo);
    let worktree_path = repo_root.join(Path::new(worktree_name));

    // When deleting a worktree the HEAD still remains in refs/heads.
    // git, when invoked directly, will account for its potential existence when creating a
    // worktree; whereas when invoked through git2 (or via libgit2) does not seem to account for it
    // and will throw an error.
    // This means that using redwood to create, then delete, and then recreate a worktree using the
    // same name would fail when using git2, so we invoke git directly instead for this.
    let mut cmd = std::process::Command::new("git");
    cmd.args(["worktree", "add", worktree_path.to_str().unwrap()]);
    return match cmd.output() {
        Ok(_) => Ok(()),
        Err(e) => Err(RedwoodError::CommandError {
            command: format!("{:?}", cmd),
            message: e.to_string(),
        }),
    };
}

pub fn get_repo_root(repo: &Repository) -> PathBuf {
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

    return repo_root;
}

pub fn open_repo(path: &Path) -> Result<Repository> {
    match Repository::open(path) {
        Ok(repo) => Ok(repo),
        Err(err) => return Err(RedwoodError::from(err)),
    }
}

pub fn prune_worktree(repo: &Repository, worktree_name: &str) -> Result<()> {
    let worktree = match repo.find_worktree(worktree_name) {
        Ok(wt) => wt,
        Err(e) => return Err(RedwoodError::from(e)),
    };

    if let Err(e) = worktree.prune(Some(
        WorktreePruneOptions::new().valid(true).working_tree(true),
    )) {
        return Err(RedwoodError::from(e));
    }

    Ok(())
}

pub fn find_worktree(repo: &Repository, worktree_name: &str) -> Result<git2::Worktree> {
    match repo.find_worktree(worktree_name) {
        Ok(wt) => Ok(wt),
        Err(err) => return Err(RedwoodError::from(err)),
    }
}
