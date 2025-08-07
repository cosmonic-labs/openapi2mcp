# Example OpenAPI Specifications

This directory contains example OpenAPI specifications for testing the `openapi2mcp` tool.

## Available Examples

### 1. Pet Store API (`petstore.yaml`)
A comprehensive pet store API example featuring:
- Complete CRUD operations for pets
- Path parameters and query parameters
- Request body validation with schemas
- Component references (`$ref`)
- Multiple HTTP methods (GET, POST, PUT, DELETE)
- Complex data types with validation rules

**Test it:**
```bash
cargo run -- -i examples/petstore.yaml -o output/petstore -t typescript
```

### 2. Simple Task API (`simple-api.json`)
A minimal task management API in JSON format featuring:
- Basic CRUD operations
- Query parameters for filtering
- Simple request/response schemas
- PATCH method for partial updates
- Inline schema definitions

**Test it:**
```bash
cargo run -- -i examples/simple-api.json -o output/tasks -t rust -n task-server
```

### 3. Weather API (`weather-api.yaml`)
A weather information API featuring:
- Multiple endpoints for different data types
- Query parameters with validation
- Enum values for parameter options
- Date formatting
- Forecast arrays

**Test it:**
```bash
cargo run -- -i examples/weather-api.yaml -o output/weather -t typescript -n weather-mcp
```

### 4. GitHub API (`github-api.yaml`)
A subset of GitHub's REST API featuring:
- Authentication with bearer tokens
- Security schemes and OAuth
- Repository management operations
- Issue tracking functionality
- Complex nested schemas with references
- Path parameters and query filtering

**Test it:**
```bash
cargo run -- -i examples/github-api.yaml -o output/github -t rust -n github-server
```

### 5. E-commerce API (`ecommerce-api.yaml`)
A comprehensive e-commerce platform API featuring:
- Product catalog management
- Order processing and tracking
- Customer management
- Multiple authentication methods (API key + OAuth2)
- Complex filtering and pagination
- Nested object schemas
- UUID-based identifiers

**Test it:**
```bash
cargo run -- -i examples/ecommerce-api.yaml -o output/ecommerce -t typescript -n ecommerce-mcp
```

### 6. Slack API (`slack-api.yaml`)
A subset of Slack's Web API featuring:
- Channel and conversation management
- Message sending and history
- User information retrieval
- Bearer token authentication
- Form-encoded request bodies
- Complex nested response structures
- Pagination with cursors

**Test it:**
```bash
cargo run -- -i examples/slack-api.yaml -o output/slack -t rust -n slack-server
```

## Testing the Examples

You can use these examples to test various features of the tool:

1. **Different input formats**: JSON vs YAML
2. **Various OpenAPI features**: parameters, request bodies, responses
3. **Complex vs simple schemas**: references vs inline definitions
4. **Different HTTP methods**: GET, POST, PUT, DELETE, PATCH
5. **Output targets**: TypeScript vs Rust generation

## Expected Output

Each example should generate:
- **TypeScript target**: Node.js project with MCP SDK integration
- **Rust target**: Cargo project with MCP server implementation

The generated servers will include:
- Tool definitions for each API operation
- Input schema validation
- Placeholder implementation code
- All necessary dependencies and configuration