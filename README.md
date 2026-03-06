# retool (`rt`)

A `kubectx`-style CLI for switching between named skill profiles for AI agents.

## Motivation

Managing agent skills across different workflows — personal dev, agent testing, code review — means manually swapping directories in and out of `~/.agents/skills`. `rt` replaces that with a single command. 

## How it works

`~/.agents/skills` is a symlink pointing to the active profile directory under `~/.agents/profiles/`. Switching profiles replaces that symlink. By default, `rt` also updates `~/.claude/skills` so Claude Code picks up the same profile automatically.

```
~/.agents/
  profiles/
    coding/
      git-helper/
      code-review/
    agent-testing/
      deploy-skill/
      test-runner/
  skills -> ~/.agents/profiles/coding/   # active profile
  .retool_previous                        # tracks previous profile for toggle

~/.claude/
  skills -> ~/.agents/profiles/coding/   # kept in sync by rt
```

## Installation

```bash
git clone https://github.com/mglea/retool
cd retool
cargo install --path .
```

## Usage

```bash
rt                        # List all profiles; active one is highlighted
rt <name>                 # Switch to a profile (updates both ~/.agents/skills and ~/.claude/skills)
rt -                      # Switch to previous profile (toggle)
rt skills                 # List skills in the current profile
rt create <name> [path]   # Snapshot skills into a new profile
rt -d <name>              # Delete a profile
rt --help                 # Show help
rt --version              # Show version

# Opt out of updating one symlink target:
rt <name> --no-claude     # Only update ~/.agents/skills
rt <name> --no-agents     # Only update ~/.claude/skills
```

## Example

```bash
# Create profiles from existing skill directories
rt create coding ~/.agents/skills
rt create agent-testing ~/some/other/skills

# Switch between them
rt coding
rt agent-testing

# Toggle back
rt -

```
