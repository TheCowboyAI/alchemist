#!/bin/bash

# Replace all complex event conversion blocks with simple acknowledgments
file="cim-domain-workflow/src/handlers/workflow_command_handler.rs"

# This is a complex replacement, so I'll use perl for better multiline handling
perl -i -pe '
  BEGIN { $in_block = 0 }
  if (/Convert events/) { $in_block = 1; $_ = "" }
  elsif ($in_block && /serde_json::to_value.*\)\)/) { 
    $_ = "        Ok(CommandAcknowledgment {\n            command_id: envelope.id,\n            correlation_id: envelope.correlation_id,\n            status: CommandStatus::Accepted,\n            reason: None,\n        })\n";
    $in_block = 0;
  }
  elsif ($in_block) { $_ = "" }
' "$file" 