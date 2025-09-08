use openapiv3::OpenAPI;

mod converter;

pub struct MCPServer {
    pub tools: Vec<Tool>,
    // pub auth_stuff: AuthStuff,
}

pub struct Tool {
    pub calls: Vec<Call>,
}

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
