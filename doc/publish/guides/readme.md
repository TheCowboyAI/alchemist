# CIM Developer Guides

Practical guides for working with the Composable Information Machine.

## Getting Started

### [Quick Start Guide](./getting-started.md)
Get up and running with CIM in 15 minutes.
- Setting up your development environment
- Running the example application
- Basic graph operations
- First workflow creation

### [Development Setup](./development-setup.md)
Complete NixOS development environment setup.
- Installing Nix
- Using the development shell
- NATS server configuration
- IDE setup (VSCode/Cursor)

## Core Guides

### [Event-Driven Testing](./event-driven-testing.md)
Testing framework for event-driven systems.
- Writing event sequence tests
- Using EventStreamValidator
- Testing cross-domain workflows
- Mocking NATS for unit tests

### [Domain Development](./domain-development.md)
Creating new domain modules.
- Domain structure and patterns
- Implementing aggregates
- Defining events and commands
- Cross-domain communication

### [Bevy Integration](./bevy-integration.md)
Working with the UI layer.
- Understanding the async/sync bridge
- Creating visual components
- Handling user interactions
- Real-time event updates

## Advanced Guides

### [Performance Optimization](./performance-optimization.md)
Tuning CIM for production.
- NATS JetStream configuration
- Event batching strategies
- Projection optimization
- Memory management

### [Security Configuration](./security-configuration.md)
Securing your CIM deployment.
- NATS authentication setup
- JWT token configuration
- Subject-based permissions
- Encryption at rest

### [Deployment Guide](./deployment-guide.md)
Running CIM in production.
- NixOS deployment
- Docker containerization
- Kubernetes orchestration
- Monitoring and observability

## Integration Guides

### [GitHub Integration](./github-integration.md)
Using the Git domain with GitHub.
- MCP GitHub tool setup
- Repository analysis
- Commit graph visualization
- Automated workflows

### [AI Agent Integration](./ai-agent-integration.md)
Connecting AI systems to CIM.
- Agent registration
- Capability management
- Tool access control
- Conversation handling

## Troubleshooting

### [Common Issues](./troubleshooting.md)
Solutions to frequent problems.
- NATS connection issues
- Event ordering problems
- Performance bottlenecks
- Memory leaks

### [Debug Techniques](./debug-techniques.md)
Advanced debugging strategies.
- Event stream inspection
- Time-travel debugging
- Performance profiling
- Distributed tracing

## Best Practices

### [Event Design Patterns](./event-design-patterns.md)
Designing effective domain events.
- Event granularity
- Naming conventions
- Schema evolution
- Versioning strategies

### [CQRS Implementation](./cqrs-implementation.md)
Command and query separation.
- Command validation
- Aggregate design
- Projection strategies
- Consistency boundaries

## Quick Links

- **New to CIM?** Start with [Getting Started](./getting-started.md)
- **Building a domain?** See [Domain Development](./domain-development.md)
- **Testing your code?** Check [Event-Driven Testing](./event-driven-testing.md)
- **Ready for production?** Read [Deployment Guide](./deployment-guide.md)

## Need Help?

- Check the [FAQ](./faq.md)
- Review [Common Issues](./troubleshooting.md)
- Join our [Community Discord](https://discord.gg/cim)
- File an [Issue on GitHub](https://github.com/TheCowboyAI/alchemist/issues) 