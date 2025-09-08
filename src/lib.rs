use openapiv3::OpenAPI;

use crate::mcp_server::MCPServer;

pub mod codegen_typescript;
pub mod mcp_server;

pub fn openapi2mcp(openapi: OpenAPI) -> anyhow::Result<MCPServer> {
    mcp_server::MCPServer::from_openapi(openapi)
}
