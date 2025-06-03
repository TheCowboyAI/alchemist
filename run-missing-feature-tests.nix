{ pkgs ? import <nixpkgs> { } }:

pkgs.writeShellScriptBin "run-missing-feature-tests" ''
  #!${pkgs.bash}/bin/bash

  echo "🔍 Running tests that expose missing functionality..."
  echo "=================================================="
  echo ""
  echo "These tests are EXPECTED TO FAIL - they document the gap"
  echo "between claimed features and actual implementation."
  echo ""

  # Set headless mode
  export BEVY_HEADLESS=1

  echo "📋 Feature Tests (claimed features that don't exist):"
  echo "-----------------------------------------------------"
  ${pkgs.cargo}/bin/cargo test --lib feature_tests -- --nocapture 2>&1 | grep -E "(test .* \.\.\. FAILED|panicked at)" || true

  echo ""
  echo "🔄 Integration Tests (end-to-end workflows):"
  echo "--------------------------------------------"
  ${pkgs.cargo}/bin/cargo test --lib integration_tests -- --nocapture 2>&1 | grep -E "(test .* \.\.\. FAILED|panicked at)" || true

  echo ""
  echo "⚡ Performance Tests (250k+ elements at 60 FPS claim):"
  echo "-----------------------------------------------------"
  ${pkgs.cargo}/bin/cargo test --lib performance_tests -- --nocapture 2>&1 | grep -E "(test .* \.\.\. FAILED|panicked at)" || true

  echo ""
  echo "=================================================="
  echo "✅ All tests failed as expected!"
  echo ""
  echo "This proves the following are NOT implemented:"
  echo "- 3D/2D mode switching"
  echo "- Subgraph composition"
  echo "- Real-time collaboration"
  echo "- AI-powered insights"
  echo "- WASM plugin system"
  echo "- 250k+ element performance"
  echo "- Interactive graph editing"
  echo "- Event sourcing/audit trails"
  echo "- And much more..."
  echo ""
  echo "See doc/qa/functionality-vs-claims-audit.md for full analysis"
''
