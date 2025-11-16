# Contributing to slack-rs

Thank you for your interest in contributing to slack-rs! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please be respectful and professional in all interactions.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)
- Git

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/slack-rs.git
   cd slack-rs
   ```

3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/slackapi/slack-rs.git
   ```

4. Create a new branch for your feature or bug fix:
   ```bash
   git checkout -b feature/my-new-feature
   ```

## Development Workflow

### Building the Project

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features
```

### Running Tests

```bash
# Run all tests
cargo test --all-features --workspace

# Run tests for a specific module
cargo test --lib audit_logs

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

We maintain high code quality standards. Before submitting a pull request, ensure your code passes all checks:

```bash
# Format your code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run Clippy (linter)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all-features --workspace
```

### Documentation

All public APIs must have documentation. Generate and check the documentation:

```bash
# Generate documentation
cargo doc --no-deps --open

# Test documentation examples
cargo test --doc
```

## Testing Approach

This project follows a Test-Driven Development (TDD) approach:

1. **Write tests first**: Before implementing a feature, write tests that define the expected behavior
2. **Mirror Python SDK tests**: When adding features from the Python SDK, port the corresponding tests first
3. **Comprehensive coverage**: Aim for 85%+ test coverage
4. **Unit and integration tests**: Include both unit tests for individual components and integration tests for complete flows

### Test Organization

- Unit tests: Place in the same file as the code being tested (in a `#[cfg(test)]` mod tests` block)
- Integration tests: Place in the `tests/` directory
- Test naming: Use descriptive names that explain what is being tested

## Pull Request Process

1. **Update tests**: Ensure your changes are covered by tests
2. **Update documentation**: Update relevant documentation and examples
3. **Run quality checks**: Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass
4. **Commit message format**: Use clear, descriptive commit messages:
   ```
   Add feature: Brief description

   Longer description if needed, explaining:
   - Why this change is necessary
   - How it addresses the issue
   - Any breaking changes

   Fixes #123
   ```

5. **Create the PR**:
   - Fill out the pull request template completely
   - Reference any related issues
   - Provide clear description of changes
   - Include examples if applicable

6. **Code Review**:
   - Address reviewer feedback promptly
   - Keep discussions focused and professional
   - Be open to suggestions and alternative approaches

## Project Structure

```
slack-rs/
├── benches/           # Benchmark tests
├── examples/          # Example applications
├── src/
│   ├── audit_logs/    # Audit Logs API
│   ├── http_retry/    # HTTP retry logic
│   ├── models/        # Block Kit and data models
│   ├── oauth/         # OAuth implementation
│   ├── scim/          # SCIM API
│   ├── signature/     # Signature verification
│   ├── socket_mode/   # Socket Mode client
│   ├── web/           # Web API client
│   └── webhook/       # Webhook clients
├── tests/             # Integration tests
└── Cargo.toml         # Project configuration
```

## Feature Development Guidelines

### Adding New API Methods

1. Check the Python SDK for the corresponding method
2. Port the tests first
3. Implement the method signature in the client
4. Implement the method logic
5. Ensure all tests pass
6. Add documentation with examples

### Adding New Models

1. Port the model structure from the Python SDK
2. Add serde serialization/deserialization
3. Add validation logic if needed
4. Write comprehensive tests
5. Document all fields

### Performance Considerations

- Use `&str` instead of `String` for function parameters when possible
- Avoid unnecessary allocations
- Use `tokio` for async operations
- Consider using `Arc` for shared data in async contexts

## Common Tasks

### Adding a New Dependency

1. Add the dependency to `Cargo.toml`
2. Justify the addition in your PR description
3. Ensure it's compatible with the project's license (MIT)

### Updating Documentation

- Update `README.md` for user-facing changes
- Update module-level documentation (`//!`) for API changes
- Update `CHANGELOG.md` for notable changes

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Ask questions in pull requests
- Check the documentation and examples first

## License

By contributing to slack-rs, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in the project's documentation and releases. Thank you for helping make slack-rs better!
