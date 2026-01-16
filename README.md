# OpenAPI to MCP Server Generator

Generate MCP server tools from OpenAPI specifications. Works with [mcp-server-template-ts](https://github.com/cosmonic-labs/mcp-server-template-ts).

## Usage

### Install via npm

```shell
npm install -g openapi2mcp
```

Or run directly with npx:

```shell
npx openapi2mcp <spec.yaml> --project-path <output-dir>
```

### Generate an MCP Server

Start with an MCP project using our template:

```shell
git clone https://github.com/cosmonic-labs/mcp-server-template-ts.git my-mcp-server
```

Generate MCP tools into the server project from an OpenAPI specification:

```shell
openapi2mcp path/to/openapi.yaml --project-path my-mcp-server
```

### CLI Options

| Option | Description |
|--------|-------------|
| `--project-path <path>` | Path to the project root directory (default: `.`) |
| `--include-tools <regex>` | Regex pattern for tools to include |
| `--include-methods <methods>` | Comma-separated HTTP methods to include (e.g., `GET,POST`) |
| `--skip-long-tool-names` | Skip tools with names exceeding max length instead of erroring |
| `--oauth2` | Enable OAuth2 authentication |
| `--oauth2-auth-url <url>` | OAuth2 authorization URL (required if `--oauth2` is set) |
| `--oauth2-token-url <url>` | OAuth2 token URL (required if `--oauth2` is set) |
| `--oauth2-refresh-url <url>` | OAuth2 refresh token URL |

### Example with Options

```shell
openapi2mcp api-spec.yaml \
  --project-path ./my-server \
  --include-methods GET,POST \
  --include-tools "users|products" \
  --oauth2 \
  --oauth2-auth-url "https://auth.example.com/authorize" \
  --oauth2-token-url "https://auth.example.com/token"
```

## Building from Source

### Prerequisites

- Rust toolchain with `wasm32-wasip2` target
- Node.js 18+
- [jco](https://github.com/bytecodealliance/jco)

### Build

```shell
npm install
npm run build
```

This compiles the Rust code to WASM and transpiles it to JavaScript.

### Run from Source

```shell
git clone https://github.com/cosmonic-labs/mcp-server-template-ts.git tests/petstore/generated

node index.js tests/petstore/input.json --project-path tests/petstore/generated
```
