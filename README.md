# pj

<p align="center">
    <a href="https://crates.io/crates/pj-cli">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/pj-cli.svg"></a>
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
- **AI Assistant Integration**: Optionally launch codex, claude, gemini, or a custom AI assistant when opening a project
- **Multiplexer Support**: Automatically opens vim/nvim in a split pane and renames the tmux window to the project name when running inside tmux or cmux
- **Git Integration**: Prompt to initialize git repositories when adding projects
- **GitHub Integration**: Optionally create GitHub remotes via the gh CLI

## Installation

### Homebrew

```bash
brew install alceal/tap/pj
```

### Cargo

```bash
cargo install pj-cli
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
pj --init
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
| `pj <filter>` | Filter projects by terms (auto-selects if single match) |
| `pj --init` | Run the setup wizard |
| `pj --config` | Interactive configuration editor |
| `pj --list` | Display all tracked projects with status |
| `pj -a` / `pj --add` | Add current directory as a project |
| `pj --rm` | Remove projects interactively |
| `pj --rm-missing` | Remove all projects with missing paths |

### Options

| Option | Description |
|--------|-------------|
| `-t, --tags <TAGS>` | Add tags to project(s) or filter when adding |
| `--rm-tags <TAGS>` | Remove tags from project(s) |
| `-e, --editor <EDITOR>` | Override the configured editor |
| `--no-editor` | Skip opening editor (just cd if enabled) |
| `--ai <AI_ASSISTANT>` | Override the configured AI assistant |
| `--no-ai` | Skip opening AI assistant |
| `--cd` / `--no-cd` | Override directory change behavior |
| `--non-interactive` | Disable prompts and fuzzy picker; auto-select highest-frecency match |

### Examples

```bash
# Add project with multiple tags
pj -a -t work/backend,rust,api

# Filter projects by name (auto-selects if single match)
pj rust
pj my-project

# Open project with a different editor
pj -e zed

# Open project with an AI assistant
pj --ai claude

# List all tracked projects
pj --list

# Remove all projects with missing paths
pj --rm-missing

# Edit configuration interactively
pj --config
```

## Non-Interactive Mode

For scripts, CI, editor integrations, and any context without a human at the
terminal, pj can suppress all prompts and the fuzzy picker. Activate via the
`--non-interactive` flag, the `PJ_NON_INTERACTIVE=1` environment variable, or
automatically when stderr is not a TTY (e.g., when pj's stderr is redirected).
Precedence: explicit flag > environment variable > stderr auto-detection.

Behavior per command:

- `pj <filter>` auto-selects the highest-frecency existing match. If no
  existing project matches, exits with an error.
- `pj` (no filter) auto-selects the highest-frecency existing project.
- `pj --init` writes a config from built-in defaults and installs the shell
  function (no wizard).
- `pj -a` skips the git-init prompt (uses `git_init_on_add` as the answer)
  and the GitHub visibility prompt (defaults to **Private**).
- `pj --rm <filter>` removes every matching project directly. `pj --rm`
  without a filter errors instead of opening the multi-select picker.
- `pj -t <tags> <path>` requires both `--tags` and an explicit path.
- `pj --config` is unavailable; edit `~/.pj/config.toml` directly or run
  `pj --init --non-interactive` to regenerate defaults.

Truthy values for `PJ_NON_INTERACTIVE`: `1`, `true`, `TRUE`, `yes`, `YES`.

## Configuration

Configuration is stored in `~/.pj/config.toml`:

```toml
editor = "code"           # Editor command to launch
cd_on_select = true       # Change directory when selecting a project
git_init_on_add = true    # Prompt to initialize git when adding a project
gh_create_on_add = false  # Create GitHub remote when adding (requires gh CLI)
ai_assistant = "none"     # AI assistant to launch (none, codex, claude, gemini, or custom command)
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

# Filter by tag in selection
pj work           # Fuzzy matches against path and tags
pj work/frontend  # More specific filter
```

## License

This project is licensed under the MIT License. See the LICENSE.txt file for
details.
