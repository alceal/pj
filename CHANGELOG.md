# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-01-05

### Added

- STATUS column in `pj --list` output showing whether project paths exist (OK/MISSING)

## [0.2.1] - 2026-01-01

### Changed

- Updated README documentation for v0.2.0 CLI changes

## [0.2.0] - 2026-01-01

### Changed

- CLI restructured to use flags instead of subcommands (`pj --list` instead of `pj list`)
- Skim selection now fullscreen (100% height)
- Filter terms passed as arguments for smart matching (`pj rust` filters projects)
- Auto-select when filter results in single match

### Added

- Interactive configuration editor (`pj --config`) with arrow key navigation
- Multi-select instructions header in skim screens
- Esc key support to cancel init wizard
- New editor options: `none`, `nano`, `emacs`

## [0.1.2] - 2025-12-29

### Fixed

- Shell function now correctly handles non-directory output (`--help`, `--version`, `list`)
- Use `builtin cd` in bash/zsh to avoid conflicts with cd-wrapping plugins (e.g., enhancd)

## [0.1.1] - 2025-12-26

### Changed

- Rename crate to `pj-cli` for crates.io (binary name remains `pj`)

## [0.1.0] - 2025-12-26

### Added

- Interactive fuzzy project selector powered by skim
- Frecency-based ranking algorithm combining frequency and recency
- Hierarchical tag system with `/` separator support
- Shell integration for bash, zsh, fish, and POSIX sh
- Editor integration with configurable editor command
- Git auto-initialization when adding projects
- GitHub remote creation via gh CLI (opt-in)
- Interactive setup wizard (`pj init`)
- Commands:
  - `pj` - Select and open a project
  - `pj -a` - Add current directory as a project
  - `pj list` - Display all tracked projects in a table
  - `pj tag` - Manage project tags
  - `pj rm` - Remove projects from tracking
  - `pj rm --missing` - Bulk remove projects with missing paths
- Configuration via `~/.pj/config.toml`
- Project data storage in `~/.pj/projects.json`
