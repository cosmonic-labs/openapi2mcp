use std::collections::HashMap;

use http::Method;
use openapiv3::{OpenAPI, ReferenceOr};

use crate::mcp_server::{
    Call, MCPServer, MCPTool, MCPToolProperty, MCPToolPropertyType, PropertyId, ValueSource,
};

pub fn openapi_to_mcp_server(openapi: OpenAPI) -> anyhow::Result<MCPServer> {
    let mut tools = Vec::new();
    for (path, path_item_ref) in &openapi.paths.paths {
        // Handle ReferenceOr for path items
        let path_item = match path_item_ref {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { reference } => {
                log::error!("Path item reference not supported: {}", reference);
                return Err(anyhow::anyhow!(
                    "Path item references are not supported: {}",
                    reference
                ));
            }
        };

        if let Some(operation) = &path_item.get {
            tools.push(operation_to_tool(Method::GET, path, operation)?);
            log::info!("Added GET tool for path: {}", path);
        }
        if let Some(operation) = &path_item.post {
            tools.push(operation_to_tool(Method::POST, path, operation)?);
            log::info!("Added POST tool for path: {}", path);
        }
        if let Some(operation) = &path_item.put {
            tools.push(operation_to_tool(Method::PUT, path, operation)?);
            log::info!("Added PUT tool for path: {}", path);
        }
        if let Some(operation) = &path_item.delete {
            tools.push(operation_to_tool(Method::DELETE, path, operation)?);
            log::info!("Added DELETE tool for path: {}", path);
        }
        if let Some(operation) = &path_item.patch {
            tools.push(operation_to_tool(Method::PATCH, path, operation)?);
            log::info!("Added PATCH tool for path: {}", path);
        }
    }

    log::info!("Created {} MCP tools", tools.len());

    // TODO: handle multiple servers
    assert!(openapi.servers.len() <= 1);
    let base_url = openapi
        .servers
        .first()
        .map(|s| s.url.clone())
        .unwrap_or(String::new());
    Ok(MCPServer {
        name: openapi.info.title,
        version: openapi.info.version,
        description: openapi.info.description,
        tools,
        base_url,
    })
}

fn operation_to_tool(
    method: Method,
    path: &str,
    operation: &openapiv3::Operation,
) -> anyhow::Result<MCPTool> {
    let tool_name = format!(
        "{}_{}",
        method.to_string().to_lowercase(),
        path.trim_matches('/')
            .replace(',', "_")
            .replace('/', "_")
            .replace('{', "")
            .replace('}', "")
    );

    let description = operation
        .description
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    let mut headers = HashMap::new();
    let mut query = HashMap::new();
    let mut properties = Vec::new();

    // TODO: take another look at the parameters
    for param_ref in &operation.parameters {
        match param_ref {
            // ReferenceOr::Reference { reference } => todo!(),
            ReferenceOr::Reference { reference } => {}
            ReferenceOr::Item(param) => match param {
                openapiv3::Parameter::Query {
                    parameter_data,
                    allow_reserved,
                    style,
                    allow_empty_value,
                } => {
                    query.insert(
                        parameter_data.name.clone(),
                        ValueSource::Property(PropertyId::from_query(&parameter_data.name)),
                    );
                    properties.push(MCPToolProperty {
                        name: parameter_data.name.clone(),
                        description: parameter_data.description.clone(),
                        required: parameter_data.required,
                        // TODO: don't hardcode string
                        type_: MCPToolPropertyType::String,
                    });
                }
                openapiv3::Parameter::Header {
                    parameter_data,
                    style,
                } => {
                    headers.insert(
                        parameter_data.name.clone(),
                        ValueSource::Property(PropertyId::from_header(&parameter_data.name)),
                    );
                    properties.push(MCPToolProperty {
                        name: parameter_data.name.clone(),
                        description: parameter_data.description.clone(),
                        required: parameter_data.required,
                        // TODO: don't hardcode string
                        type_: MCPToolPropertyType::String,
                    });
                }
                // openapiv3::Parameter::Path {
                //     parameter_data,
                //     style,
                // } => todo!(),
                // openapiv3::Parameter::Cookie {
                //     parameter_data,
                //     style,
                // } => todo!(),
                _ => {}
            },
        }
    }

    Ok(MCPTool {
        // TODO: allow multiple calls
        calls: vec![Call {
            method,
            path: path.to_string(),
            headers,
            query,
            body: None,
        }],
        properties,
        name: tool_name,
        description,
        // name: tool_name,
        // description,
        // input_schema,
    })
}
