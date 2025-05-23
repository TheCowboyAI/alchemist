{ pkgs ? import <nixpkgs> { } }:

let
  # Import our cache configuration
  cacheConfig = import ./cache-config.nix;

  # Default package for testing cache
  dummyPackage = pkgs.runCommand "alchemist-dummy" { } ''
    echo "Alchemist cache test package"
    date > $out
  '';

  # Check if a path exists in the cache
  # Returns true if it exists, false otherwise
  pathExistsInCache = path:
    let
      basename = builtins.baseNameOf path;
      result = builtins.tryEval (
        builtins.fetchurl {
          url = "${cacheConfig.localCache}/${basename}.narinfo";
          name = "check-${basename}";
        }
      );
    in
    result.success;

  # Get git repo information
  getGitInfo = pkgs.runCommand "git-info"
    {
      buildInputs = [ pkgs.git ];
    } ''
    cd ${toString ./.}
    echo "Git Status:" > $out
    git status --porcelain >> $out
    echo "\nCurrent Hash:" >> $out
    git rev-parse HEAD >> $out
    echo "\nCurrent Branch:" >> $out
    git rev-parse --abbrev-ref HEAD >> $out
    echo "\nDirty Status:" >> $out
    if git diff-index --quiet HEAD --; then
      echo "clean" >> $out
    else
      echo "dirty" >> $out
    fi
  '';

  # Function to create a checksum of all source files, useful for cache invalidation
  getSourceChecksum = pkgs.runCommand "source-checksum"
    {
      buildInputs = [ pkgs.nix ];
    } ''
    cd ${toString ./.}
    echo "Source Files Hash:" > $out
    find ./src -type f -name "*.rs" | sort | xargs sha256sum | sha256sum | cut -d ' ' -f 1 >> $out
    echo "\nCargo.toml Hash:" >> $out
    sha256sum Cargo.toml | cut -d ' ' -f 1 >> $out
    echo "\nCargo.lock Hash:" >> $out
    sha256sum Cargo.lock | cut -d ' ' -f 1 >> $out
  '';

  # Analyze derivation for cacheability issues
  analyzeCacheability = pkg: pkgs.runCommand "cache-analysis"
    {
      buildInputs = [ pkgs.nix ];
    } ''
    echo "Cache Analysis for ${pkg.name}:" > $out
    echo "\nDerivation Path:" >> $out
    nix show-derivation ${pkg.drvPath} | head -20 >> $out
    echo "\nPossible Cache Invalidation Factors:" >> $out
    
    # Check if the derivation has __impure attribute
    if grep -q "__impure" ${pkg.drvPath}; then
      echo "WARNING: Derivation marked as impure (__impure attribute)" >> $out
    fi
    
    # Check for timestamps or volatile attributes
    if grep -q "$(date +%Y)" ${pkg.drvPath}; then
      echo "WARNING: Derivation contains current year timestamp" >> $out
    fi
    
    # Check for Git dirty status inclusion
    if cd ${toString ./.} && ! git diff-index --quiet HEAD --; then
      echo "WARNING: Git working directory is dirty" >> $out
      echo "This will add unique timestamp to Nix store path and prevent cache hits" >> $out
    fi
    
    # Check nix-hash
    echo "\nNix Hash:" >> $out
    nix-hash ${pkg.drvPath} >> $out
    
    echo "\nRealizable:" >> $out
    if nix path-info --store ${cacheConfig.localCache} ${pkg.outPath} 2>/dev/null; then
      echo "✅ Available in local cache" >> $out
    else
      echo "❌ Not available in local cache" >> $out
    fi

    echo "\nRecommendation:" >> $out
    if grep -q "WARNING" $out; then
      echo "Fix the warnings above to improve cacheability" >> $out
      echo "Consider using a clean Git state: just build-from-commit" >> $out
    else
      echo "Derivation looks cacheable. If not in cache, add with: nix copy --to ${cacheConfig.localCache} ${pkg.outPath}" >> $out
    fi
  '';

  # Get all build dependencies of a derivation
  getDeps = pkg: pkgs.runCommand "build-deps"
    {
      buildInputs = [ pkgs.nix ];
    } ''
    echo "Build Dependencies:" > $out
    nix-store -qR ${pkg.drvPath} >> $out
    
    echo "\nDirect Dependencies:" >> $out
    nix-store -q --references ${pkg.drvPath} >> $out
    
    echo "\nRuntime Dependencies:" >> $out
    nix-store -q --references ${pkg.outPath} >> $out
  '';

  # Create a script that can be used to verify all deps are in cache
  verifyDepsScript = pkg: pkgs.writeScriptBin "verify-deps" ''
    #!/bin/sh
    echo "Checking if all dependencies are in cache..."
    MISSING=0
    TOTAL=0
    
    for dep in $(nix-store -qR ${pkg.drvPath}); do
      TOTAL=$((TOTAL+1))
      if ! nix path-info --store ${cacheConfig.localCache} $dep &>/dev/null; then
        echo "❌ Missing: $dep"
        MISSING=$((MISSING+1))
      fi
    done
    
    if [ $MISSING -eq 0 ]; then
      echo "✅ All $TOTAL dependencies available in cache"
    else
      echo "❌ $MISSING/$TOTAL dependencies missing from cache"
    fi
  '';

  # Create a local cache report
  cacheReport = pkgs.writeScriptBin "cache-report" ''
    #!/bin/sh
    echo "Alchemist Cache Report"
    echo "======================"
    echo "Cache URL: ${cacheConfig.localCache}"
    echo "Cache Key: ${cacheConfig.localCacheKey}"
    echo ""
    
    echo "Checking cache connectivity..."
    if curl -s -I ${cacheConfig.localCache}/nix-cache-info >/dev/null; then
      echo "✅ Cache server is reachable"
    else
      echo "❌ Cannot connect to cache server at ${cacheConfig.localCache}"
      exit 1
    fi
    
    echo ""
    echo "Checking for current build in cache..."
    if nix path-info --store ${cacheConfig.localCache} -r .#default 2>/dev/null; then
      echo "✅ Current build is available in cache"
    else
      echo "❌ Current build is not in cache"
    fi
    
    echo ""
    echo "Git Status:"
    if cd ${toString ./.} && git diff-index --quiet HEAD --; then
      echo "✅ Git workspace is clean"
    else
      echo "❌ Git workspace is dirty (will prevent cache hits)"
      echo "   Run 'git status' to see changes"
      echo "   Consider using 'just build-from-commit' to use a clean state"
    fi
    
    echo ""
    echo "Global Nix Configuration:"
    echo "------------------------"
    nix show-config | grep -E 'substituter|key'
  '';

  # Create an environment for cache testing
  cacheEnv = pkgs.buildEnv {
    name = "alchemist-cache-env";
    paths = [
      dummyPackage
      cacheReport
      (verifyDepsScript dummyPackage)
    ];
  };

in
{
  inherit dummyPackage cacheReport pathExistsInCache getGitInfo getSourceChecksum analyzeCacheability getDeps verifyDepsScript cacheEnv;
}
