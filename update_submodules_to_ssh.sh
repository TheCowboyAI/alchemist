#!/bin/bash

# Script to update all submodule URLs from HTTPS to SSH

echo "Updating submodule URLs to use SSH..."

# Update each submodule's URL in git config
git submodule foreach '
    echo "Processing $name..."
    current_url=$(git config --get remote.origin.url)
    
    if [[ $current_url == https://github.com/* ]]; then
        # Convert HTTPS to SSH format
        new_url=$(echo $current_url | sed "s|https://github.com/|git@github.com:|")
        echo "  Changing from: $current_url"
        echo "  Changing to:   $new_url"
        git remote set-url origin $new_url
    else
        echo "  Already using SSH or non-GitHub URL: $current_url"
    fi
'

echo -e "\nUpdating .gitmodules file..."

# Update the .gitmodules file
sed -i 's|https://github.com/|git@github.com:|g' .gitmodules

echo -e "\nSyncing submodule configuration..."
git submodule sync

echo -e "\nDone! All submodules have been updated to use SSH."
echo "Don't forget to commit the .gitmodules changes."