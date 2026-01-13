use std::{
    collections::BTreeMap,
    fmt::{self, Display},
};

use http::Method;
use openapiv3::{AuthorizationCodeOAuth2Flow, OpenAPI};

pub use converter::ConverterOptions;

mod converter;

#[derive(Debug, Clone)]
pub struct MCPServer {
    pub name: String,
    pub tools: Vec<MCPTool>,
    pub version: String,
    pub description: Option<String>,
    pub base_url: String,
    pub oauth2_info: Option<AuthorizationCodeOAuth2Flow>,
}

#[derive(Debug, Clone)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    // pub required: bool,
    pub properties: Vec<MCPToolProperty>,
    pub call: Call,
}

#[derive(Debug, Clone)]
pub struct MCPToolProperty {
    pub name: String,
    pub description: Option<String>,
    pub required: MCPToolPropertyRequired,
    pub type_: MCPToolPropertyType,
}

#[derive(Debug, Clone)]
pub enum MCPToolPropertyRequired {
    Optional,
    Required,
    Default(serde_json::Value), // TODO: use self::Value?
}

#[derive(Debug, Clone)]
pub struct PropertyId(String);

impl Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum MCPToolPropertyType {
    String,
    Number,
    Boolean,
    Array(Box<MCPToolProperty>),
    Object(BTreeMap<String, MCPToolProperty>),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub method: Method,
    pub headers: BTreeMap<String, ValueSource>,
    pub path: String,
    pub path_params: BTreeMap<String, ValueSource>,
    pub query: BTreeMap<String, ValueSource>,
    pub body: Option<ValueSource>,
}

#[derive(Debug, Clone)]
pub enum ValueSource {
    Fixed(Value),
    Property(PropertyId),
    // Auth
}

impl Display for ValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueSource::Fixed(value) => write!(f, "{}", value),
            ValueSource::Property(property) => write!(f, "{}", property),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    // TODO: should we have multiple number types?
    Number(f64),
    Boolean(bool),
    // Object(HashMap<String, Value>),
    // Array(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(value) => write!(f, "\"{}\"", value),
            Value::Number(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
        }
    }
}

impl MCPServer {
    pub fn from_openapi(openapi: OpenAPI, options: ConverterOptions) -> anyhow::Result<Self> {
        converter::openapi_to_mcp_server(openapi, options)
    }
}

impl PropertyId {
    pub fn from_header(header: &str) -> Self {
        // Self(format!("header-{}", header))
        Self(header.to_string())
    }

    pub fn from_query(query: &str) -> Self {
        // Self(format!("query-{}", query))
        Self(query.to_string())
    }

    pub fn from_path(path: &str) -> Self {
        // Self(format!("path-{}", path))
        Self(path.to_string())
    }

    // pub fn from_cookie(cookie: &str) -> Self {
    //     Self(format!("cookie-{}", cookie))
    // }

    pub fn from_body(body: &str) -> Self {
        // Self(format!("path-{}", path))
        Self(body.to_string())
    }
}
