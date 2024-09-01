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

## Shell auto-completion

### Zsh

```shell
make install-zsh-completions
```

### Fish

Copy this to your fish config file:

```fish
# Redwood completions
set -l rw_commands delete help list new open version
complete -c redwood --no-files -n "not __fish_seen_subcommand_from $rw_commands"\
	-a 'delete help list new open version'
complete -c redwood -n "__fish_seen_subcommand_from open" \
	-a "(redwood list)"
complete -c redwood -n "__fish_seen_subcommand_from delete" \
    -a "(redwood list --only-worktrees)"
complete -c redwood -n "__fish_seen_subcommand_from new" \
    -a "(redwood list --only-bare-repos)"
```
