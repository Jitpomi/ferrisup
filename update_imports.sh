#!/bin/bash
# Script to update all imports from 'shared' to 'ferrisup-common'
find ./ferrisup/src -type f -name "*.rs" -exec sed -i '' 's/use shared/use ferrisup_common/g' {} \;
