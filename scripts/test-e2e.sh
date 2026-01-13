#!/bin/bash
set -euo pipefail

# E2E test script for openapi2mcp
# Generates MCP servers from OpenAPI specs and validates they build
# NOTE: This script requires `wash` CLI to be installed and configured
# NOTE: Assumes you have already built the openapi2mcp plugin to wasm

# Cleanup function
cleanup() {
	echo "Cleaning up..."
	wash plugin uninstall openapi2mcp || true
	rm -rf ./tests/*/generated
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

echo "Starting E2E tests for openapi2mcp..."

# Check if the wasm file exists
WASM_FILE="./target/wasm32-wasip2/release/openapi2mcp.wasm"
if [ ! -f "$WASM_FILE" ]; then
	echo "Error: Wasm file not found at $WASM_FILE"
	exit 1
fi

# Install the plugin
echo "Installing openapi2mcp wash plugin..."
if ! wash plugin install "$WASM_FILE" --force; then
	echo "Error: Failed to add openapi2mcp as a wash plugin"
	exit 1
fi

# Process each test directory
test_count=0
success_count=0

for dir in ./tests/*/; do
	if [ ! -d "$dir" ]; then
		continue
	fi

	# Remove trailing slash to normalize path
	dir=${dir%/}

	test_count=$((test_count + 1))
	echo ""
	echo "Processing test $test_count: $dir"

	# Look for spec files in the test directory
	spec_file=""
	if [ -f "$dir/input.yaml" ]; then
		spec_file="$dir/input.yaml"
		echo "Found YAML spec: $spec_file"
	elif [ -f "$dir/input.json" ]; then
		spec_file="$dir/input.json"
		echo "Found JSON spec: $spec_file"
	else
		echo "Error: No input.yaml or input.json found in $dir"
		exit 1
	fi

	# Generate MCP server code
	echo "Generating MCP server from $spec_file..."
	wash new ${PWD}/mcp-server-template-ts --name "$dir/generated"

	# if this is microsoft-graph then its too large, so we need to skip some routes
	if [ "$dir" == "./tests/microsoft-graph" ]; then
		wash openapi2mcp "$spec_file" --project-path "$dir/generated" --include-methods GET --include-tools "drives/\\{drive-id\\}|me/drive|me/mail|me/calendar|me/chats" --tool-name-exceeded-action Skip --oauth2 true --oauth2-auth-url "https://login.microsoftonline.com/common/oauth2/v2.0/authorize" --oauth2-token-url "https://login.microsoftonline.com/common/oauth2/v2.0/token"
	else
		wash openapi2mcp "$spec_file" --project-path "$dir/generated"
	fi

	if [ $? -ne 0 ]; then
		echo "Error: Failed to generate MCP server from $spec_file"
		echo "Check if the OpenAPI spec is valid and the plugin is working"
		exit 1
	fi

	if [ ! -d "$dir/generated/src/routes/v1/mcp/tools" ]; then
		echo "Error: Generated project missing tools directory in $dir/generated/src/routes/v1/mcp/tools"
		exit 1
	fi

	# Build the generated project
	echo "Building generated project in $dir/generated"
	(
		cd "$dir/generated" || {
			echo "Error: Failed to cd into $dir/generated"
			exit 1
		}
		if ! wash build; then
			echo "Error: Failed to build project in $dir/generated"
			echo "Check the generated TypeScript code for syntax errors"
			exit 1
		fi
	) || exit 1

	# Check dist/component.wasm exists
	if [ ! -f "$dir/generated/dist/component.wasm" ]; then
		echo "Error: Build succeeded but component.wasm not found in $dir/generated/dist"
		exit 1
	fi

	success_count=$((success_count + 1))
	echo "✓ Test $test_count completed successfully"
done

echo ""
echo "E2E tests completed: $success_count/$test_count tests passed"

if [ $success_count -eq $test_count ]; then
	echo "All tests passed! 🎉"
	exit 0
else
	echo "Some tests failed 😞"
	exit 1
fi
