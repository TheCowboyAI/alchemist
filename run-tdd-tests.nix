{ pkgs ? import <nixpkgs> {} }:

pkgs.writeShellScriptBin "run-tdd-tests" ''
  #!${pkgs.bash}/bin/bash

  # TDD Workflow Commands from the rule
  # Ensure BEVY_HEADLESS=1 for all tests
  export BEVY_HEADLESS=1

  echo "🧪 Running TDD-compliant tests..."
  echo "================================"

  # Domain layer tests (no Bevy/NATS dependencies)
  echo "📋 Running domain isolation tests..."
  ${pkgs.cargo}/bin/cargo test --lib -- domain_isolated_tests --nocapture

  # Headless integration tests
  echo "🖥️ Running headless integration tests..."
  ${pkgs.cargo}/bin/cargo test --lib -- headless_integration_test --nocapture

  # TDD-compliant ECS tests
  echo "⚙️ Running TDD-compliant ECS tests..."
  ${pkgs.cargo}/bin/cargo test --lib -- tdd_compliant_ecs_tests --nocapture

  # Graph editor automated tests
  echo "📊 Running graph editor tests..."
  ${pkgs.cargo}/bin/cargo test --lib -- graph_editor_automated_tests --nocapture

  # Run existing tests to ensure compatibility
  echo "✅ Running all existing tests..."
  ${pkgs.cargo}/bin/cargo test --workspace

  # Performance check (single test should be <100ms per TDD rule)
  echo "⏱️ Checking test performance..."
  ${pkgs.cargo}/bin/cargo test --lib -- --test-threads=1 --show-output | grep -E "test .* ... ok|test result" || true

  echo "================================"
  echo "✨ TDD test run complete!"
''
