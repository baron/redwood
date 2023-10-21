# Redwood

Redwood is a git worktree+tmux management tool. The fundamental issue it solves
is being able to create a new git worktree and tmux session in one command, and
later recreate the session inside the worktree.

## Installation

### Build from source

```shell
sudo make install
```

### Pre-compiled binaries

Pre-compiled binaries are available in the releases section.

## Configuration

By default redwood will iterate over all directories in the home directory
(except for git worktree directories) to find git repositories. Since there are
often a lot of deeply nested folders in home directories it is recommended to
configure redwood to ignore some folders that are not relevant and commonly
known to be larget (e.g. `node_modules`, `cargo`, `target` etc). This can be
done by setting the environment variable `REDWOOD_IGNORED_DIRS`, which is
expected to be a comma-separated string where each element matches a directory
name to ignore. For example:

```shell
export REDWOOD_IGNORED_DIRS="node_modules,target,.git,.cargo,.rustup,go"
```
