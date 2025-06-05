{ lib
, pkgs
, nonRustDeps
}:

# Import the base package
let
  basePackage = import ./package.nix {
    inherit lib pkgs nonRustDeps;
  };
in
# Override to enable tests
basePackage.overrideAttrs (oldAttrs: {
  pname = "ia-with-tests";

  # Enable tests
  doCheck = true;

  # Test environment variables
  BEVY_HEADLESS = "1";
  RUST_BACKTRACE = "1";

  # Additional test dependencies if needed
  checkInputs = with pkgs; [
    # Add any test-specific dependencies here
  ];
})
