use openapiv3::OpenAPI;

mod converter;

#[derive(Debug, Clone)]
pub struct MCPServer {
    pub tools: Vec<MCPTool>,
    // pub version: String,
    // pub description: String,
    // pub auth_stuff: AuthStuff,
}

#[derive(Debug, Clone)]
pub struct MCPTool {
    // pub name: String,
    // pub description: String,
    pub calls: Vec<Call>,
}

#[derive(Debug, Clone)]
pub struct Call {
    // call it ApiClient?
    pub method: String,
    pub path: String,
    // pub body: Option<Body>,
    // pub headers: Option<Headers>,
    // pub query: Option<Query>,
    // pub cookie: Option<Cookie>,
}

// struct AuthStuff {
//     ...
// }

impl MCPServer {
    pub fn from_openapi(openapi: OpenAPI) -> anyhow::Result<Self> {
        converter::openapi_to_mcp_server(openapi)
    }
}
