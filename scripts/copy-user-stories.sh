#!/bin/bash

# Script to copy user stories from doc/testing to individual domains

echo "Copying user stories to domain directories..."

# Define mappings
declare -A STORY_MAPPINGS=(
    ["agent"]="agent"
    ["conceptualspaces"]="conceptualspaces"
    ["document"]="document"
    ["location"]="location"
    ["organization"]="organization"
    ["person"]="person"
    ["policy"]="policy"
    ["workflow"]="workflow"
)

# Copy user stories
for domain in "${!STORY_MAPPINGS[@]}"; do
    src_file="doc/testing/user-stories-${STORY_MAPPINGS[$domain]}-domain.md"
    dest_dir="cim-domain-${domain}/doc"
    
    if [ -f "$src_file" ]; then
        echo "Copying $src_file to $dest_dir/user-stories.md"
        mkdir -p "$dest_dir"
        cp "$src_file" "$dest_dir/user-stories.md"
    else
        echo "Warning: $src_file not found"
    fi
done

# Special cases
echo "Handling special cases..."

# cim-ipld already has user stories
if [ -f "cim-ipld/docs/USER_STORIES.md" ]; then
    echo "cim-ipld already has user stories at cim-ipld/docs/USER_STORIES.md"
fi

# cim-domain-person already has user stories
if [ -f "cim-domain-person/docs/user_stories.md" ]; then
    echo "cim-domain-person already has user stories at cim-domain-person/docs/user_stories.md"
    # But let's copy to the standard location too
    mkdir -p "cim-domain-person/doc"
    cp "cim-domain-person/docs/user_stories.md" "cim-domain-person/doc/user-stories.md"
fi

echo "User stories copy complete!" 