# Redwood

Redwood is a git worktree+tmux management tool. The fundamental issue it solves
is being able to create a new git worktree and tmux session in one command, and
later recreate the session inside the worktree.

## Usage

### Create new session

```shell
redwood new <path_to_git_repo> <worktree_name>
```

`path_to_git_repo` resolves to the root of the git directory in which it
exists. Using `.` is allowed.

`worktree_name` is used as the name for both the worktree, branch and tmux
session.

### Open previous session

```shell
redwood open <worktree_name>
```

### List worktrees

```shell
redwood list
```

### Delete worktree

```shell
redwood delete <worktree_name>
```

## Installation

```shell
sudo make install
```

## Configuration

Redwood searches for the configuration file first at
`XDG_CONFIG_HOME/redwood/conf.json`, then `$HOME/redwood/conf.json`.
