#!/bin/bash

# Start from the current directory
BASE_DIR=$(pwd)

echo "Starting cargo clean in subdirectories containing Cargo.toml..."

# Find all Cargo.toml files and clean their directories
find "$BASE_DIR" -type f -name "Cargo.toml" | while read -r cargo_file; do
    project_dir=$(dirname "$cargo_file")
    echo "Running 'cargo clean' in $project_dir"
    (cd "$project_dir" && cargo clean)
done

echo "Finished cleaning all projects."
