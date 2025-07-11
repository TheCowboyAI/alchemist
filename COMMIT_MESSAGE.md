feat: Implement complete Iced UI system for Dashboard and Dialog

This commit adds comprehensive UI functionality using Iced, providing both
system monitoring (Dashboard) and AI conversation (Dialog) interfaces.

## Dashboard Implementation
- Real-time system monitoring with memory usage and uptime tracking
- Interactive domain view with clickable elements and detail panels  
- NATS event streaming integration for live updates
- Dark theme with responsive design

## Dialog UI Implementation
- Full-featured AI chat interface with multiple model support
- Real-time token streaming display
- Export functionality (Markdown, JSON, text)
- Message history with user/AI distinction

## Architecture Changes
- Pure event-based communication using channels
- Command/Event pattern for UI-backend separation
- In-process window implementation for development
- Unified ALCHEMIST-EVENTS stream to resolve conflicts

## Supporting Infrastructure
- System monitor for resource tracking
- Dialog handler connecting UI to AI providers
- Comprehensive test suite with real API tests
- Complete documentation and examples

## Test Coverage
- AI integration tests with real API keys
- Workflow execution tests with real commands
- UI functionality test scripts
- Automated test runner

## Documentation
- UI Guide with architecture details
- Quick Start guide for new users
- Troubleshooting guide for common issues
- Examples showing custom UI creation

This implementation fulfills the requirement for an Iced-based UI that is
"ready for a dev to work inside of" with all communication through events,
compatible with both TEA (The Elm Architecture) and ECS (Entity Component
System) patterns.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>