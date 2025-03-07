#!/bin/bash

# Check if a commit message was provided
if [ -z "$1" ]; then
    echo "Usage: $0 \"Your commit message\""
    exit 1
fi

# Move to the correct directory (relative to the script location)
cd "$(dirname "$0")/../liz-desktop-bin/" || { echo "Failed to cd into package directory"; exit 1; }

# Update checksums
echo "Updating checksums..."
updpkgsums

# Generate .SRCINFO
echo "Generating .SRCINFO..."
makepkg --printsrcinfo > .SRCINFO

# Show current git status
echo "Current Git Status:"
git status

# Ask user for confirmation
read -p "Do you want to continue? (Y/n): " confirm
confirm=${confirm,,} # Convert to lowercase

if [[ "$confirm" == "n" ]]; then
    echo "Operation canceled."
    exit 1
fi

# Add changes to git
echo "Staging changes..."
git add .

# Commit with user-provided message
echo "Committing changes..."
git commit -m "$1"

# Push to AUR repository
echo "Pushing to AUR..."
git push

echo "Done!"

