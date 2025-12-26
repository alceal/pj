# pj

<p align="center">
    <a href="https://crates.io/crates/pj">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/pj.svg"></a>
    <a href="https://github.com/alceal/pj/blob/main/LICENSE.txt">
    <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg">
    </a>
</p>

A fast, terminal-based project launcher with fuzzy matching. Quickly navigate
between projects, organize them with tags, and launch your editor with a single
command.

## Features

- **Fuzzy Matching**: Interactive TUI with real-time filtering powered by skim
- **Frecency Ranking**: Smart ordering based on frequency and recency of access
- **Hierarchical Tags**: Organize projects with nested tags (e.g., `work/backend`)
- **Shell Integration**: Automatic directory changing for bash, zsh, fish, and sh
- **Editor Integration**: Launch your preferred editor when selecting a project
- **Git Integration**: Auto-initialize git repositories when adding projects
- **GitHub Integration**: Optionally create GitHub remotes via the gh CLI

## Installation

### Homebrew

```bash
brew tap alceal/tap
brew install pj
```

### Cargo

```bash
cargo install pj
```

### From Source

```bash
git clone https://github.com/alceal/pj
cd pj
cargo install --path .
```

## Quick Start

Run the interactive setup wizard:

```bash
pj init
```

This will configure your shell, editor, and preferences. The wizard detects
your shell automatically and installs the necessary shell function for
directory changing.

Add your first project:

```bash
cd ~/projects/my-app
pj -a -t work,rust
```

Select and open a project:

```bash
pj
```

## Commands

| Command | Description |
|---------|-------------|
| `pj` | Open interactive project selector |
| `pj init` | Run the setup wizard |
| `pj -a` | Add current directory as a project |
| `pj list` | Display all tracked projects |
| `pj tag` | Manage project tags |
| `pj rm` | Remove projects from tracking |

### Options

| Option | Description |
|--------|-------------|
| `-t, --tags <TAGS>` | Filter by tags or add tags when using `-a` |
| `-e, --editor <EDITOR>` | Override the configured editor |
| `--cd` / `--no-cd` | Override directory change behavior |

### Examples

```bash
# Add project with multiple tags
pj -a -t work/backend,rust,api

# Filter projects by tag
pj -t work

# List projects with a specific tag
pj list -t rust

# Open project with a different editor
pj -e zed

# Remove all projects with missing paths
pj rm --missing
```

## Configuration

Configuration is stored in `~/.pj/config.toml`:

```toml
editor = "code"           # Editor command to launch
cd_on_select = true       # Change directory when selecting a project
git_init_on_add = true    # Initialize git when adding a project
gh_create_on_add = false  # Create GitHub remote when adding (requires gh CLI)
```

Project data is stored in `~/.pj/projects.json`.

## Shell Integration

When `cd_on_select` is enabled, pj installs a shell function that wraps the
binary to enable directory changing. The setup wizard handles this
automatically for:

- **Bash**: Added to `~/.bashrc`
- **Zsh**: Added to `~/.zshrc`
- **Fish**: Added to `~/.config/fish/config.fish`
- **POSIX sh**: Added to `~/.profile`

## Tag Hierarchy

Tags support hierarchical organization using `/` as a separator:

```bash
# Add with hierarchical tags
pj -a -t work/frontend/dashboard

# Filter matches parent and all children
pj -t work           # Matches work, work/frontend, work/frontend/dashboard
pj -t work/frontend  # Matches work/frontend and work/frontend/dashboard
```

## License

This project is licensed under the MIT License. See the LICENSE.txt file for
details.
