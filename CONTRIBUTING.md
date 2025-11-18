# Contributing to rapid-rs

Thank you for your interest in contributing to rapid-rs! This is an early-stage project, and we welcome all contributions.

## How to Contribute

### Reporting Bugs

Open an issue on GitHub with:
- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version, etc.)

### Suggesting Features

Open an issue with:
- Clear description of the feature
- Use cases and motivation
- Possible implementation approach

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo clippy`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Development Setup

```bash
git clone https://github.com/ashishjsharda/rapid-rs
cd rapid-rs
cargo build
cargo test

# Run the example
cd examples/rest-api
cargo run
```

## Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add documentation comments for public APIs

## Testing

- Write tests for new features
- Ensure all tests pass before submitting PR
- Include integration tests where appropriate

## Questions?

Feel free to open an issue or reach out to [@ashishjsharda](https://github.com/ashishjsharda)

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.
