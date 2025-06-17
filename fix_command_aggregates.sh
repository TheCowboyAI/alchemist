#!/bin/bash

# Fix workflow commands
sed -i '/impl Command for/a\    type Aggregate = Workflow;\n' cim-domain-workflow/src/commands/workflow_commands.rs

# Fix step commands  
sed -i '/impl Command for/a\    type Aggregate = Workflow;\n' cim-domain-workflow/src/commands/step_commands.rs

echo "Fixed all Command trait implementations" 