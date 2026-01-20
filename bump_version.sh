#!/bin/bash

# Script to update version in Cargo.toml and package.json
# Usage: ./bump_version.sh [new_version] or ./bump_version.sh [major|minor|patch]

set -e

CARGO_FILE="Cargo.toml"
PACKAGE_FILE="package.json"

# Function to get current version from Cargo.toml
get_current_version() {
    grep -E '^version = ' "$CARGO_FILE" | sed -E 's/^version = "([^"]+)".*/\1/'
}

# Function to get version from package.json
get_package_version() {
    grep -E '"version":' "$PACKAGE_FILE" | sed -E 's/.*"version": "([^"]+)".*/\1/'
}

# Function to validate version was updated correctly
validate_version() {
    local expected_version=$1
    local cargo_version=$(get_current_version)
    local package_version=$(get_package_version)
    
    if [ "$cargo_version" != "$expected_version" ]; then
        echo "Error: Version in $CARGO_FILE is '$cargo_version', expected '$expected_version'"
        return 1
    fi
    
    if [ "$package_version" != "$expected_version" ]; then
        echo "Error: Version in $PACKAGE_FILE is '$package_version', expected '$expected_version'"
        return 1
    fi
    
    return 0
}

# Function to update version in Cargo.toml
update_cargo_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_FILE"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_FILE"
    fi
}

# Function to update version in package.json
update_package_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" "$PACKAGE_FILE"
    else
        # Linux
        sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" "$PACKAGE_FILE"
    fi
}

# Function to increment version
increment_version() {
    local version=$1
    local part=$2  # major, minor, or patch
    
    IFS='.' read -ra ADDR <<< "$version"
    local major=${ADDR[0]}
    local minor=${ADDR[1]}
    local patch=${ADDR[2]}
    
    case $part in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Main logic
if [ $# -eq 0 ]; then
    echo "Usage: $0 [new_version] or $0 [major|minor|patch]"
    echo "Example: $0 1.0.0"
    echo "Example: $0 patch"
    exit 1
fi

current_version=$(get_current_version)
echo "Current version: $current_version"

if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    # Direct version provided
    new_version=$1
elif [[ "$1" =~ ^(major|minor|patch)$ ]]; then
    # Increment version
    new_version=$(increment_version "$current_version" "$1")
    echo "Incrementing $1 version..."
else
    echo "Error: Invalid version format or increment type"
    echo "Version must be in format X.Y.Z (e.g., 1.0.0) or one of: major, minor, patch"
    exit 1
fi

echo "New version: $new_version"

# Update both files
update_cargo_version "$new_version"
update_package_version "$new_version"

# Validate that the version was updated correctly
if ! validate_version "$new_version"; then
    echo "Error: Version validation failed. Please check the files manually."
    exit 1
fi

# Regenerate Cargo.lock and
cargo check

# Regenerate package-lock.json
npm i --package-lock-only

# Print the new version
echo "✓ Updated version to $new_version"
