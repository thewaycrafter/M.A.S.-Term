# Contributing to MASTerm

Thank you for your interest in contributing to MASTerm! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Development Setup

```bash
# Clone the repository
git clone https://github.com/masterm-dev/masterm.git
cd masterm

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

## Project Structure

```
masterm/
├── crates/
│   ├── masterm-core/      # Core engine library
│   ├── masterm-cli/       # CLI binary
│   ├── masterm-shell/     # Shell adapters
│   └── masterm-plugins/   # Built-in plugins
├── docs/                  # Documentation
└── tests/                 # Integration tests
```

## Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit with a descriptive message
8. Push and create a Pull Request

## Code Style

- Follow Rust idioms and best practices
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add documentation for public APIs
- Write tests for new functionality

## Plugin Development

See [docs/plugins.md](docs/plugins.md) for the plugin development guide.

## Commit Messages

Use conventional commits:
- `feat: Add new feature`
- `fix: Fix bug`
- `docs: Update documentation`
- `refactor: Refactor code`
- `test: Add tests`
- `chore: Maintenance tasks`

## Pull Request Process

1. Update documentation if needed
2. Add tests for new functionality
3. Ensure all tests pass
4. Update CHANGELOG.md if applicable
5. Request review from maintainers

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

## Questions?

- Open an issue for bugs or feature requests
- Join our Discord for discussions
- Check existing issues before creating new ones

Thank you for contributing! 🚀
