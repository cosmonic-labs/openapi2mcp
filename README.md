# OpenAPI to MCP Server Generator

## Usage

### Run as a CLI

```shell
cargo run -- -i [path/to/open/yaml/or/json] -o [path/to/generated/mcp/server]
```

### Use as a [wash](https://github.com/cosmonic-labs/wash) plugin

Compile targeting WASIp2
```shell
cargo build --target wasm32-wasip2 --release
```

Install as `wash` plugin
```shell
wash plugin install ./target/wasm32-wasip2/release/openapi2mcp.wasm
```

Run as `wash` plugin
```shell
wash openapi2mcp [path/to/open/yaml/or/json] --dir [path/to/generated/mcp/server]
```
