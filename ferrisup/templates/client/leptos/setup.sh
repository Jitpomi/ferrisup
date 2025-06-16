#!/bin/bash

# This script is executed when a user selects the Leptos client_old framework
# It prompts the user to select a specific Leptos template and then creates the project

echo "🦀 FerrisUp Leptos Template Selector"
echo "Select a Leptos template:"
echo "1) Counter - Simple counter with reactive state management"
echo "2) Router - Multi-page application with client-side navigation"
echo "3) Todo - Todo application with filtering capabilities"
echo "4) SSR - Server-side rendered application with Axum"
echo "5) Full-Stack - Complete application with API endpoints and server functions"

read -p "Enter your choice (1-5): " choice

# Get the project directory from the first argument
PROJECT_DIR=$1
TEMPLATE=""

case $choice in
    1)
        TEMPLATE="counter"
        ;;
    2)
        TEMPLATE="router"
        ;;
    3)
        TEMPLATE="todo"
        ;;
    4)
        TEMPLATE="ssr"
        ;;
    5)
        TEMPLATE="fullstack"
        ;;
    *)
        echo "Invalid choice. Defaulting to counter template."
        TEMPLATE="counter"
        ;;
esac

# Create the project using the ferrisup command with the selected template
cd "$(dirname "$PROJECT_DIR")"
PROJECT_NAME=$(basename "$PROJECT_DIR")
rm -rf "$PROJECT_DIR"  # Remove the empty directory created by the original command

# Use the ferrisup command to create the project with the selected template
ferrisup new --template $TEMPLATE $PROJECT_NAME

echo "✅ Successfully created Leptos $TEMPLATE project in $PROJECT_DIR"
