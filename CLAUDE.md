# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the `openapi2mcp` project - a tool for converting OpenAPI specifications to MCP (Model Context Protocol) format. The repository is currently empty and needs initial implementation.

## Project Purpose

Based on the repository name, this tool should:
- Parse OpenAPI/Swagger specifications
- Convert them to MCP (Model Context Protocol) server definitions
- Generate appropriate MCP server code or configuration
- Creates MCP servers that compile to Wasm

## Development Setup

- Programming language: Rust
- Package management and build system: Cargo
- CLI framework: clap

## Common Commands

- `cargo build` - Build the project
- `cargo run -- --help` - Show CLI help
- `cargo run -- -i spec.yaml -o output/` - Convert OpenAPI spec to MCP server
- `cargo run -- -i spec.yaml -o output/ -l rust` - Convert OpenAPI spec to Rust MCP server
- `cargo test` - Run tests
- `cargo check` - Check code without building
- `cargo fmt` - Format code
- `cargo clippy` - Run linting
- `cargo audit` - Security audit

## Project Structure

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Library exports
- `src/cli.rs` - Command line interface using clap
- `src/error.rs` - Error types and handling
- `src/openapi.rs` - OpenAPI spec parsing and validation
- `src/mcp.rs` - MCP server generation logic

## Architecture

The tool follows a pipeline architecture:
1. **CLI parsing** - Parse command line arguments and configuration
2. **OpenAPI parsing** - Load and validate OpenAPI specifications (JSON/YAML)
3. **MCP conversion** - Transform OpenAPI operations into MCP tools
4. **Code generation** - Generate TypeScript or Rust MCP server projects

## Supported Features

- OpenAPI 3.x specification parsing (JSON/YAML)
- Multiple output targets (TypeScript, Rust)
- CLI interface with input/output path configuration
- Complete validation of OpenAPI specs and error handling
- Generation of complete MCP server projects with dependencies
- TypeScript projects use official @modelcontextprotocol/sdk
- Rust projects provide structured placeholder with guidance for MCP SDK integration
- Comprehensive test coverage (24 unit tests + 14 integration tests)
- 6 example OpenAPI specifications covering diverse use cases

## Generated Output Structure

### TypeScript Projects
- `package.json` with MCP SDK dependencies
- `tsconfig.json` with proper TypeScript configuration  
- `src/index.ts` with complete MCP server implementation
- All OpenAPI operations converted to MCP tools with proper schemas

### Rust Projects  
- `Cargo.toml` with guidance for MCP SDK selection (rmcp or rust-mcp-sdk)
- `src/main.rs` with structured server implementation
- Tool definitions with proper schema documentation
- Placeholder implementations ready for API integration
- Clear TODO comments for completion steps

## Testing

The project includes comprehensive test coverage:

### Unit Tests (24 tests)
- Error handling and type conversion tests
- OpenAPI parsing and validation tests  
- MCP server generation tests
- CLI argument parsing tests
- Schema transformation tests

### Integration Tests (14 tests)
- End-to-end CLI functionality testing
- All example OpenAPI specifications
- Both TypeScript and Rust target generation
- Error handling for invalid inputs
- Generated code compilation verification
- CLI help and version output validation

Run tests with:
```bash
cargo test           # All tests
cargo test --lib     # Unit tests only
cargo test --test integration_tests  # Integration tests only
```

## Example OpenAPI Specifications

The `examples/` directory contains 6 comprehensive OpenAPI specifications:

1. **Simple Task API** (`simple-api.json`) - Basic CRUD operations
2. **Pet Store API** (`petstore.yaml`) - Comprehensive REST API with references
3. **Weather API** (`weather-api.yaml`) - Multiple endpoints with parameters
4. **GitHub API** (`github-api.yaml`) - Authentication, complex schemas, path parameters
5. **E-commerce API** (`ecommerce-api.yaml`) - Full e-commerce platform with OAuth2
6. **Slack API** (`slack-api.yaml`) - Real-world API with form-encoded requests

Each example demonstrates different OpenAPI features and complexity levels.

## CI/CD Pipeline

The project uses GitHub Actions for comprehensive CI/CD:

### Continuous Integration (`ci.yml`)
- **Multi-Rust Testing**: Stable, beta, and nightly versions
- **Code Quality**: Format checks, linting with clippy
- **Cross-Platform**: Linux, Windows, macOS builds
- **Security**: Dependency auditing with cargo-audit
- **Coverage**: Code coverage reporting with codecov
- **Example Validation**: Tests all example specifications

### Release Automation (`release.yml`)
- **Triggered by**: Version tags (e.g., `v1.0.0`)
- **Multi-Platform Binaries**: Linux, macOS, Windows
- **GitHub Releases**: Automatic release creation
- **Cargo Publishing**: Publishes to crates.io

### PR Validation (`pr-validation.yml`)
- **Generated Code Testing**: Validates TypeScript/Rust compilation
- **Breaking Changes**: Detects API changes with cargo-semver-checks
- **Performance**: Benchmarks parsing and generation speed

### Additional Workflows
- **Auto-labeling**: Smart PR/issue labeling based on changes
- **Dependabot**: Automated dependency updates
- **Security**: Regular vulnerability scanning

All workflows use the `main` branch (never `master`).

## Notes

- This project is in the cosmonic-labs organization, suggesting it may integrate with Cosmonic's wasmCloud ecosystem
- Consider MCP specification compliance when implementing
- Should support common OpenAPI versions (3.0, 3.1)
- Use a template repository depending on the language for the MCP Server chosen, for Typescript use https://github.com/cosmonic-labs/mcp-server-template-ts and we will create on for rust soon.
