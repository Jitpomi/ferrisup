#!/bin/bash
# Test script to debug shared component creation

# Clean up any previous test
rm -rf test_app

# Create a new server app
echo "Creating a new server app..."
cargo run -- new test_app server poem

# Try to transform it and add a shared component
echo "Transforming the app and adding a shared component..."
# shellcheck disable=SC2164
cd test_app
cargo run -- transform
