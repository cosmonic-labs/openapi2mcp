# openapi2mcp

A Rust CLI tool that converts OpenAPI specifications to MCP (Model Context Protocol) servers.

## Features

- Parse OpenAPI 3.x specifications (JSON/YAML)
- Generate MCP servers in TypeScript or Rust
- CLI interface with flexible configuration
- Complete project scaffolding with dependencies

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Convert OpenAPI spec to TypeScript MCP server
cargo run -- -i api.yaml -o output/ -t typescript

# Convert to Rust MCP server
cargo run -- -i api.json -o output/ -t rust -n my-api-server

# Show help
cargo run -- --help
```

## Options

- `-i, --input <FILE>` - Path to OpenAPI specification file (required)
- `-o, --output <DIR>` - Output directory for generated MCP server (default: ./output)
- `-n, --name <NAME>` - Name for the generated MCP server
- `-t, --target <TARGET>` - Target language: typescript or rust (default: typescript)

## Generated Output

### TypeScript
- Complete Node.js project with package.json
- TypeScript configuration and dependencies
- MCP server implementation using @modelcontextprotocol/sdk

### Rust  
- Cargo project with dependencies
- MCP server implementation (requires mcp-sdk crate)

## Development

See [CLAUDE.md](CLAUDE.md) for development setup and commands.