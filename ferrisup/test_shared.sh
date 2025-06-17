#!/bin/bash
# Test script to debug ferrisup_common component creation

set -e  # Exit immediately if a command exits with a non-zero status

# Define colors for better output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Clean up any previous test
if [ -d "test_app" ]; then
  echo -e "${YELLOW}Removing previous test directory...${NC}"
  rm -rf test_app
fi

# Create a new server app
echo -e "${GREEN}Creating a new server app...${NC}"
ferrisup new test_app server poem

# Navigate to the test app directory
echo -e "${GREEN}Navigating to test app directory...${NC}"
cd test_app || { echo "Failed to change directory to test_app"; exit 1; }

# Transform it and add a ferrisup_common component
echo -e "${GREEN}Transforming the app and adding a shared component...${NC}"
ferrisup transform

echo -e "${GREEN}Test completed successfully!${NC}"
