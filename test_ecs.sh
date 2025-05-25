#!/usr/bin/env bash

echo "Testing ECS Architecture Implementation"
echo "======================================"

# Check if the build succeeded
if [ -f "result/bin/alchemist" ]; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Check module structure
echo ""
echo "Checking module structure:"
for module in components resources events systems bundles; do
    if [ -d "src/$module" ] && [ -f "src/$module/mod.rs" ]; then
        echo "✅ $module module exists"
    else
        echo "❌ $module module missing"
    fi
done

# Check system organization
echo ""
echo "Checking system organization:"
for system in graph rendering camera ui io; do
    if [ -d "src/systems/$system" ] && [ -f "src/systems/$system/mod.rs" ]; then
        echo "✅ systems/$system exists"
    else
        echo "❌ systems/$system missing"
    fi
done

# Check graph systems
echo ""
echo "Checking graph systems:"
for file in creation deletion selection movement validation algorithms; do
    if [ -f "src/systems/graph/$file.rs" ]; then
        echo "✅ graph/$file.rs exists"
    else
        echo "❌ graph/$file.rs missing"
    fi
done

echo ""
echo "ECS Architecture Test Complete!"
