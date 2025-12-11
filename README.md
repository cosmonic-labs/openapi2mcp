# OpenAPI to MCP Server Generator

## Usage

If you haven't already, install `wash`

```shell
curl -fsSL https://raw.githubusercontent.com/wasmcloud/wash/refs/heads/main/install.sh | bash
```

Install the latest release as a wash plugin:

```shell
wash plugin install ghcr.io/cosmonic-labs/plugins/openapi2mcp:0.5.0
```

Start with an MCP project, using our template to get started:

```shell
wash new https://github.com/cosmonic-labs/mcp-server-template-ts.git "my-mcp-server"
```
Generate MCP tools into the server project from an OpenAPI specification:

```shell
wash openapi2mcp [path/to/open/yaml/or/json] --project-path [path/to/generated/mcp/server]
```

## Building and running from source

### Run as a CLI

```shell
# cargo run -- -i [path/to/open/yaml/or/json] --project-path [path/to/generated/mcp/server]
wash new https://github.com/cosmonic-labs/mcp-server-template-ts.git "tests/petstore/generated"
cargo run -- -i tests/petstore/input.json --project-path tests/petstore/generated
```

### Use as a [wash](https://github.com/cosmonic-labs/wash) plugin

Compile to Wasm targeting WASIp2:

```shell
cargo build --target wasm32-wasip2 --release
```

Install as a `wash` plugin:

```shell
wash plugin install ./target/wasm32-wasip2/release/openapi2mcp.wasm
```

Run as `wash` plugin:

```shell
# wash openapi2mcp [path/to/open/yaml/or/json] --project-path [path/to/generated/mcp/server]
wash new https://github.com/cosmonic-labs/mcp-server-template-ts.git tests/petstore/generated
wash openapi2mcp tests/petstore/input.json --project-path tests/petstore/generated
```
