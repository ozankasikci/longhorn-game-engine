#!/bin/bash

# Script to update README badges with correct GitHub username

echo "GitHub Badge Setup for Longhorn Game Engine"
echo "=========================================="
echo

# Get GitHub username
read -p "Enter your GitHub username: " GITHUB_USERNAME

# Check if username is provided
if [ -z "$GITHUB_USERNAME" ]; then
    echo "Error: GitHub username is required"
    exit 1
fi

# Update README.md
echo "Updating README.md badges..."
sed -i.bak "s/YOUR_USERNAME/$GITHUB_USERNAME/g" README.md

echo "✅ Badges updated successfully!"
echo
echo "Next steps:"
echo "1. Sign up at https://codecov.io with your GitHub account"
echo "2. Add longhorn-game-engine repository in Codecov"
echo "3. Copy the upload token from Codecov"
echo "4. Add CODECOV_TOKEN to GitHub Secrets (Settings → Secrets → Actions)"
echo "5. Push changes to trigger CI/CD workflows"
echo
echo "For detailed instructions, see docs/CODECOV_SETUP.md"

# Clean up backup file
rm -f README.md.bak