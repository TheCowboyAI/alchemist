# Contributing to Alchemist

Thank you for your interest in contributing to Alchemist! This document provides guidelines and information for contributors.

## Copyright and Licensing

By contributing to this project, you agree that your contributions will be licensed under the MIT License and that the copyright will be assigned to Cowboy AI, LLC.

All contributions must include the following copyright notice in new files:

```rust
// Copyright (c) 2025 Cowboy AI, LLC
// Licensed under the MIT License
```

## Code of Conduct

Information Alchemist follows an **EGALITARIAN Code of Conduct** that emphasizes merit-based contribution and equal opportunity for all participants. We believe in fundamental human equality and evaluate all contributions based on their technical merit and alignment with project goals.

Please read our complete [EGALITARIAN Code of Conduct](.github/CODE_OF_CONDUCT.md) before contributing. Key principles include:

- **Merit-Based Evaluation**: All contributions are judged on technical quality and project alignment
- **Equal Voice**: Every contributor has equal opportunity to propose ideas and participate in discussions  
- **Inclusive Excellence**: High technical standards with accessible processes for all experience levels
- **Respectful Collaboration**: Professional, constructive interactions focused on the work

## Development Setup

### Prerequisites

- Rust (latest stable)
- Nix (recommended for reproducible builds)
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/alchemist
   cd alchemist
   ```
3. Set up the development environment:
   ```bash
   nix develop  # Recommended
   # OR
   cargo build
   ```

## Development Guidelines

### Architecture Principles

This project follows Domain-Driven Design (DDD) and Event-Driven Architecture (EDA) principles:

- **Domain-First**: Start with domain events and aggregates
- **Event Sourcing**: All state changes flow through immutable events
- **CQRS**: Separate command and query responsibilities
- **ECS Integration**: Use Bevy's Entity Component System for presentation

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` to catch common issues
- Write comprehensive tests for new functionality
- Document public APIs with rustdoc comments

### Testing

- Write unit tests for domain logic
- Integration tests for system interactions
- All tests must pass before submitting PR
- Aim for high test coverage

### Commit Messages

Use conventional commit format:
```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat(graph): add node clustering algorithm

Implements k-means clustering for large graphs to improve
visualization performance.

Closes #123
```

## Submitting Changes

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes following the guidelines above
3. Add tests for new functionality
4. Update documentation as needed
5. Ensure all tests pass
6. Submit a pull request

### Pull Request Requirements

- [ ] Code follows project style guidelines
- [ ] Tests are included and passing
- [ ] Documentation is updated
- [ ] Commit messages follow conventional format
- [ ] Copyright notices are included in new files
- [ ] No breaking changes without discussion

## Project Structure

```
alchemist/
├── src/                    # Main application code
├── cim-*/                  # Domain modules (submodules)
├── doc/                    # Documentation
├── examples/               # Example code
├── tests/                  # Integration tests
├── nix/                    # Nix build configuration
└── assets/                 # Static assets
```

## Domain Modules

Each `cim-*` directory is a separate domain module with its own:
- Aggregates and entities
- Commands and events
- Handlers and projections
- Tests

When contributing to domain modules, ensure changes align with Domain-Driven Design principles.

## Getting Help

- Check existing issues and discussions
- Join our community discussions
- Reach out to maintainers for guidance

## Recognition

Contributors will be recognized in our release notes and contributor list. Significant contributions may be highlighted in project announcements.

## License

By contributing to Alchemist, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Alchemist and helping build the future of visual information systems!

*Cowboy AI, LLC - Building the Composable Information Machine* 