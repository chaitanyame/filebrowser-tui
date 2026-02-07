# Contributing to File Browser TUI

Thank you for your interest in contributing to File Browser TUI! This document provides guidelines and information about contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and constructive in all interactions.

### Our Standards

Examples of behavior that contributes to a positive environment:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

### Unacceptable Behavior

Examples of unacceptable behavior:

- Harassment, trolling, or insulting language
- Personal attacks or political discussions
- Public or private harassment
- Publishing others' private information without explicit permission
- Any other conduct which could reasonably be considered inappropriate

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When filing a bug report, include:

- **Clear title** describing the bug
- **Description** of what should happen and what actually happens
- **Steps to reproduce** the issue
- **Environment info**: OS version, Rust version, terminal type
- **Screenshots** if applicable
- **Logs** with `RUST_BACKTRACE=1` if applicable

### Suggesting Features

Feature suggestions are welcome! Please:

1. Check existing issues and PRs first
2. Describe the feature use case clearly
3. Explain why it would be useful
4. Consider if it fits the project's scope (Windows TUI file browser)
5. Be open to discussion about the implementation

### Submitting Code

1. Fork the repository
2. Create a branch for your work: `git checkout -b feature/your-feature-name`
3. Make your changes following our [Coding Standards](#coding-standards)
4. Write tests for new functionality
5. Ensure all tests pass
6. Commit your changes with a clear message
7. Push to your fork
8. Submit a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A terminal that supports true color and UTF-8

### Initial Setup

```bash
# Fork and clone the repository
git clone https://github.com/your-username/filebrowser-tui.git
cd filebrowser-tui

# Add the original repository as upstream
git remote add upstream https://github.com/original-owner/filebrowser-tui.git

# Install development dependencies
cargo install cargo-watch cargo-edit
```

### Development Workflow

```bash
# Create a feature branch
git checkout -b feature/my-feature

# Make changes and test
cargo build
make test
cargo fmt
cargo clippy -- -D warnings

# Commit your changes
git add .
git commit -m "Add my feature"

# Push to your fork
git push origin feature/my-feature
```

### Useful Commands

```bash
# Watch for changes and rebuild automatically
cargo watch -x check

# Run tests on file save
cargo watch -x test

# Format all code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run specific test
cargo test test_name
```

## Coding Standards

### Rust Style

- Follow standard Rust naming conventions
- Use `rustfmt` for code formatting
- Run `cargo clippy` and fix lints
- Prefer `&str` over `&String` for function parameters
- Use `PathBuf` for file paths that need modification
- Use `&Path` for read-only path references

### Code Organization

- Keep functions focused and under 50 lines when possible
- Use modules to organize related functionality
- Document public APIs with rustdoc comments
- Write tests for all new functionality
- Keep the binary small and fast

### Testing Guidelines

- Unit tests for individual functions
- Integration tests for file operations
- Property tests for invariants
- Snapshot tests for UI changes
- Aim for high code coverage (>80%)

### Documentation

- Document all public structs, enums, and functions
- Provide usage examples in doc comments
- Keep README up to date with new features
- Update CHANGELOG for user-facing changes

## Commit Messages

Follow these guidelines for commit messages:

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### Examples

```
feat(tabs): add ability to close last tab

Previously, closing the last tab would panic. Now it creates
a new default tab when the last one is closed.

Fixes #123

feat(search): add regex support for filename filtering

Users can now use regular expressions when searching for files
by prefixing their query with 'regex:'.

Closes #456
```

## Pull Request Process

### Before Submitting

- [ ] Your code follows our coding standards
- [ ] You have added tests for new functionality
- [ ] All tests pass locally
- [ ] You have updated documentation if needed
- [ ] Your commit messages are clear and descriptive

### Submitting the PR

1. Create a descriptive pull request title
2. Reference any related issues (e.g., "Fixes #123")
3. Describe the changes in the description
4. Link to any relevant discussion

### PR Review Process

- Address all review comments
- Add additional tests if requested
- Update documentation as needed
- Keep the PR focused and atomic

### After Merge

- Update your local main branch
- Delete your feature branch
- Consider contributing to documentation

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord/Chat** (if available): Real-time discussion

## Recognition

Contributors will be recognized in:
- AUTHORS file
- Release notes for significant contributions
- Project documentation

Thank you for contributing to File Browser TUI! 🚀
