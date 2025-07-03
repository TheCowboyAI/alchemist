#!/bin/bash

echo "Fixing remaining format string errors..."

# Fix cim-domain-organization/src/aggregate/mod.rs
sed -i 's/format!("Organization {self\.name} already exists")/format!("Organization {} already exists", self.name)/g' cim-domain-organization/src/aggregate/mod.rs
sed -i 's/format!("Invalid email format: {email}")/format!("Invalid email format: {}", email)/g' cim-domain-organization/src/aggregate/mod.rs
sed -i 's/format!("Organization with name {name} already exists")/format!("Organization with name {} already exists", name)/g' cim-domain-organization/src/aggregate/mod.rs
sed -i 's/format!("Member with email {email} already exists")/format!("Member with email {} already exists", email)/g' cim-domain-organization/src/aggregate/mod.rs

# Fix cim-domain-conceptualspaces/src/dimensions.rs
sed -i 's/format!("Value {value} is outside range {:?} for dimension '\''{self\.name}'\''")/format!("Value {} is outside range {:?} for dimension '\''{}'\''", value, self.range, self.name)/g' cim-domain-conceptualspaces/src/dimensions.rs
sed -i 's/format!("Normalized value {normalized} must be in \[0, 1\]")/format!("Normalized value {} must be in [0, 1]", normalized)/g' cim-domain-conceptualspaces/src/dimensions.rs

# Fix cim-domain-location/src/value_objects/coordinates.rs
sed -i 's/format!("location\.{self\.latitude},{self\.longitude}")/format!("location.{},{}", self.latitude, self.longitude)/g' cim-domain-location/src/value_objects/coordinates.rs
sed -i 's/format!("{self\.latitude},{self\.longitude}")/format!("{},{}", self.latitude, self.longitude)/g' cim-domain-location/src/value_objects/coordinates.rs

echo "Done fixing format strings!" 