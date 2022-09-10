# Redwood

Redwood is a git worktree+tmux management tool. The fundamental issue it solves
is being able to create a new git worktree and tmux session in one command, and
later recreate the session inside the worktree.

## Installation

```shell
sudo make install
```

## Configuration

Redwood searches for the configuration file first at
`XDG_CONFIG_HOME/redwood/conf.json`, then `$HOME/redwood/conf.json`.
