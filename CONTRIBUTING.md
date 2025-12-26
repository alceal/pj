# Contributing to pj

Thank you for your interest in contributing to pj!

## Development Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/alceal/pj
   cd pj
   ```

2. Ensure you have Rust installed (rustup recommended):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build the project:

   ```bash
   cargo build
   ```

4. Run the binary:

   ```bash
   cargo run -- --help
   ```

## Code Quality

Before submitting changes, ensure your code passes these checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

## Pull Requests

1. Fork the repository
2. Create a feature branch from `develop`:

   ```bash
   git checkout -b feature/your-feature develop
   ```

3. Make your changes
4. Ensure all checks pass (`cargo fmt`, `cargo clippy`, `cargo test`)
5. Commit with a clear message describing the change
6. Push to your fork and submit a pull request to `develop`

## Commit Messages

Write clear, concise commit messages that describe what the change does:

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Reference issues when applicable

## Reporting Issues

Please use GitHub Issues to:

- Report bugs (include steps to reproduce)
- Request features (describe the use case)
- Ask questions about the codebase

## Project Structure

```
src/
├── main.rs        # CLI parser and entry point
├── config.rs      # Configuration management
├── projects.rs    # Project store and data model
├── frecency.rs    # Frecency ranking algorithm
├── github.rs      # GitHub CLI integration
├── shell.rs       # Shell detection and integration
├── tui.rs         # Terminal UI with fuzzy selection
└── commands/      # Command implementations
    ├── mod.rs
    ├── init.rs
    ├── add.rs
    ├── select.rs
    ├── list.rs
    ├── tag.rs
    └── rm.rs
```
