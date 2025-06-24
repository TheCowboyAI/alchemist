#!/usr/bin/env bash

# Fix all context.set_variable calls that need .to_string()

cd cim-domain-workflow/tests

# Create a backup
cp workflow_user_story_tests.rs workflow_user_story_tests.rs.bak

# Fix all the context.set_variable calls that are missing .to_string()
sed -i 's/context\.set_variable("\([^"]*\)"/context.set_variable("\1".to_string()/g' workflow_user_story_tests.rs

echo "Fixed all context.set_variable calls to use .to_string()" 