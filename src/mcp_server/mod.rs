use std::collections::HashMap;

use http::Method;
use openapiv3::OpenAPI;

mod converter;

#[derive(Debug, Clone)]
pub struct MCPServer {
    pub name: String,
    pub tools: Vec<MCPTool>,
    pub version: String,
    pub description: Option<String>,
    pub base_url: String,
    // pub auth_stuff: AuthStuff,
}

#[derive(Debug, Clone)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    // pub required: bool,
    pub properties: Vec<MCPToolProperty>,
    // TODO: change to singular
    pub calls: Vec<Call>,
}

#[derive(Debug, Clone)]
pub struct MCPToolProperty {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub type_: MCPToolPropertyType,
}

#[derive(Debug, Clone)]
pub struct PropertyId(String);

#[derive(Debug, Clone)]
pub enum MCPToolPropertyType {
    String,
    Number,
    Boolean,
    // Object,
    // Array,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub method: Method,
    pub headers: HashMap<String, ValueSource>,
    pub path: String,
    pub query: HashMap<String, ValueSource>,
    pub body: Option<ValueSource>,
}

#[derive(Debug, Clone)]
pub enum ValueSource {
    Fixed(Value),
    Property(PropertyId),
    // Auth
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    // Object(HashMap<String, Value>),
    // Array(Vec<Value>),
}

// struct AuthStuff {
//     ...
// }

impl MCPServer {
    pub fn from_openapi(openapi: OpenAPI) -> anyhow::Result<Self> {
        converter::openapi_to_mcp_server(openapi)
    }
}

impl PropertyId {
    pub fn from_header(header: &str) -> Self {
        Self(format!("header-{}", header))
    }

    pub fn from_query(query: &str) -> Self {
        Self(format!("query-{}", query))
    }

    // pub fn from_path(path: &str) -> Self {
    //     Self(format!("path-{}", path))
    // }

    // pub fn from_cookie(cookie: &str) -> Self {
    //     Self(format!("cookie-{}", cookie))
    // }
}
