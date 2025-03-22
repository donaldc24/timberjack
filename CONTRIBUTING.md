# Contributing to Timber

Thank you for considering contributing to Timber! This document outlines the process for contributing to the project and how to get started.

## Getting Started

1. **Fork the Repository**
    - Fork the Timber repository on GitHub
    - Clone your fork locally: `git clone https://github.com/YOUR-USERNAME/timber.git`
    - Add the original repository as an upstream remote: `git remote add upstream https://github.com/donaldcalhoun/timber.git`

2. **Set Up Development Environment**
    - Make sure you have Rust and Cargo installed
    - Install development dependencies: `cargo build`
    - Run tests to verify your setup: `cargo test`

3. **Create a Branch**
    - Create a branch from the main branch for your work
    - Use a descriptive name: `git checkout -b feature/your-feature-name` or `fix/issue-description`

## Development Workflow

1. **Make Your Changes**
    - Write code that follows the project's style and conventions
    - Add tests for new functionality
    - Ensure all tests pass: `cargo test`
    - Format your code: `cargo fmt`
    - Run linting: `cargo clippy`

2. **Commit Your Changes**
    - Write clear, concise commit messages
    - Reference issue numbers if applicable: "Fix #123: Add new feature"
    - Keep commits focused on single changes when possible

3. **Update Your Branch**
    - Fetch upstream changes: `git fetch upstream`
    - Rebase on main: `git rebase upstream/main`
    - Resolve any conflicts that arise

4. **Submit a Pull Request**
    - Push your branch to your fork: `git push origin feature/your-feature-name`
    - Create a pull request from your fork to the main repository

## Pull Request Process

1. **Code Review**
    - Maintainers will review your code
    - Address any feedback or requested changes
    - Keep the PR updated if main branch changes

2. **Testing**
    - All tests must pass before merging
    - CI will automatically run tests on your PR
    - Additional tests may be requested for certain changes

## Style Guide

- Follow Rust's official style guide
- Use `cargo fmt` to format your code
- Address all clippy warnings: `cargo clippy`
- Keep functions focused on a single responsibility
- Use descriptive variable and function names

## License

By contributing to Timber, you agree that your contributions will be licensed under the project's MIT License.

Thank you for your contribution!