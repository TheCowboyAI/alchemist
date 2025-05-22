{ pkgs ? import <nixpkgs> {}, drvPath ? null }:

let
  cacheConfig = import ./cache-config.nix;
  
  # Function to check if a path exists in the cache
  checkInCache = path: 
    let 
      result = builtins.tryEval (
        builtins.exec ["nix" "path-info" "--store" cacheConfig.localCache path]
      );
    in result.success;
  
  # Determine the derivation to analyze
  drv = if drvPath != null 
        then drvPath
        else builtins.toString (pkgs.writeText "usage" ''
          Please provide a derivation path to analyze.
          Usage: nix-build analyze-cache-miss.nix --argstr drvPath /nix/store/hash-name.drv
        '');
  
  # Create the analysis script
  analysis = pkgs.writeScriptBin "analyze-cache-miss" ''
    #!/usr/bin/env bash
    set -e
    
    DRV="${drv}"
    
    if [[ ! -f "$DRV" || ! "$DRV" =~ \.drv$ ]]; then
      echo "Error: Not a valid derivation path: $DRV"
      echo "Usage: analyze-cache-miss /nix/store/hash-name.drv"
      exit 1
    fi
    
    echo "Analyzing cache miss for derivation: $DRV"
    echo "Cache URL: ${cacheConfig.localCache}"
    echo "--------------------------------------------------------"
    
    # Check if derivation itself is in cache
    echo "Checking if derivation is in cache..."
    if nix path-info --store ${cacheConfig.localCache} "$DRV" &>/dev/null; then
      echo "✅ Derivation exists in cache"
    else
      echo "❌ Derivation not found in cache"
    fi
    
    # Get output paths
    echo -e "\nOutput paths:"
    OUT_PATHS=$(nix-store -q --outputs "$DRV")
    for OUT_PATH in $OUT_PATHS; do
      if nix path-info --store ${cacheConfig.localCache} "$OUT_PATH" &>/dev/null; then
        echo "✅ $OUT_PATH (cached)"
      else
        echo "❌ $OUT_PATH (not cached)"
      fi
    done
    
    # Analyze specific causes for a cache miss
    echo -e "\nPossible reasons for cache miss:"
    
    # Check for __impure attribute
    if grep -q "__impure" "$DRV"; then
      echo "❌ Derivation is marked as impure (__impure attribute)"
      echo "   This prevents caching and should be avoided"
    fi
    
    # Check for timestamps or current date strings
    if grep -q "$(date +%Y-%m)" "$DRV"; then
      echo "❌ Derivation contains current date ($(date +%Y-%m))"
      echo "   This changes the derivation hash on each build"
    fi
    
    # Check for git dirty status in name
    if grep -q "dirty" "$DRV"; then
      echo "❌ Derivation name contains 'dirty', suggesting a dirty Git workspace"
      echo "   This adds a timestamp to the derivation making it uncacheable"
      echo "   Solution: Commit your changes and use 'just build-from-commit'"
    fi
    
    # Check for uniqueness in the build inputs
    echo -e "\nAnalyzing build inputs (checking for uniqueness)..."
    INPUTS=$(nix-store -q --references "$DRV")
    UNIQUE_INPUTS=$(echo "$INPUTS" | grep -v "^/nix/store/.*-source$" | sort)
    
    for INPUT in $UNIQUE_INPUTS; do
      # Exclude standard nixpkgs paths which are typically fine
      if [[ "$INPUT" =~ (bash|coreutils|gnugrep|gawk) ]]; then
        continue
      fi
      
      # Check for potential timestamp-containing inputs
      if [[ "$INPUT" =~ $(date +%Y%m%d) ]]; then
        echo "❌ Input contains today's date: $INPUT"
        echo "   This likely causes cache misses"
      fi
      
      # Check for unusual/custom paths that might be unique to this build
      if [[ "$INPUT" =~ -dirty- ]]; then
        echo "❌ Input from dirty Git workspace: $INPUT"
        echo "   This will have a unique hash every build"
      fi
      
      # Check if this input is available in the cache
      if ! nix path-info --store ${cacheConfig.localCache} "$INPUT" &>/dev/null; then
        echo "ℹ️ Input not available in cache: $INPUT"
      fi
    done
    
    echo -e "\nRecommendations:"
    echo "1. Make sure your Git workspace is clean (no uncommitted changes)"
    echo "2. Use 'just build-from-commit' to build from a clean Git state"
    echo "3. Avoid impure builds with timestamps or changing values"
    echo "4. If the inputs are already cached but the output isn't, push it with:"
    echo "   nix copy --to ${cacheConfig.localCache} <output-path>"
    echo ""
    echo "For more detailed analysis of the derivation content:"
    echo "nix show-derivation $DRV | less"
  '';
  
in analysis 