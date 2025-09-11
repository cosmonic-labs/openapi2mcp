use std::collections::HashMap;

use http::Method;
use openapiv3::{OpenAPI, Parameter, PathItem, ReferenceOr};

use crate::mcp_server::{
    Call, MCPServer, MCPTool, MCPToolProperty, MCPToolPropertyType, PropertyId, ValueSource,
};

pub fn openapi_to_mcp_server(openapi: OpenAPI) -> anyhow::Result<MCPServer> {
    let mut tools = Vec::new();
    for (path, path_item_ref) in &openapi.paths.paths {
        let path_item = resolve_path(&openapi, path_item_ref).unwrap();

        if let Some(operation) = &path_item.get {
            tools.push(operation_to_tool(
                Method::GET,
                path,
                operation,
                &path_item.parameters,
                &openapi,
            )?);
            log::info!("Added GET tool for path: {}", path);
        }
        if let Some(operation) = &path_item.post {
            tools.push(operation_to_tool(
                Method::POST,
                path,
                operation,
                &path_item.parameters,
                &openapi,
            )?);
            log::info!("Added POST tool for path: {}", path);
        }
        if let Some(operation) = &path_item.put {
            tools.push(operation_to_tool(
                Method::PUT,
                path,
                operation,
                &path_item.parameters,
                &openapi,
            )?);
            log::info!("Added PUT tool for path: {}", path);
        }
        if let Some(operation) = &path_item.delete {
            tools.push(operation_to_tool(
                Method::DELETE,
                path,
                operation,
                &path_item.parameters,
                &openapi,
            )?);
            log::info!("Added DELETE tool for path: {}", path);
        }
        if let Some(operation) = &path_item.patch {
            tools.push(operation_to_tool(
                Method::PATCH,
                path,
                operation,
                &path_item.parameters,
                &openapi,
            )?);
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
        .unwrap_or_default();
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
    file_path: &str,
    operation: &openapiv3::Operation,
    route_params: &[ReferenceOr<Parameter>],
    openapi: &OpenAPI,
) -> anyhow::Result<MCPTool> {
    let tool_name = format!(
        "{}_{}",
        method.to_string().to_lowercase(),
        file_path
            .trim_matches('/')
            .replace(',', "_")
            .replace('/', "_")
            .replace('{', "")
            .replace('}', "")
    );

    let description = operation
        .description
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, file_path));

    let mut path_params = HashMap::new();
    let mut query = HashMap::new();
    let mut headers = HashMap::new();
    let mut properties = Vec::new();
    let all_params = operation.parameters.iter().chain(route_params.iter());

    // TODO: take another look at the parameters
    for param_ref in all_params {
        match resolve_parameter(openapi, param_ref).unwrap() {
            openapiv3::Parameter::Query { parameter_data, .. } => {
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
            openapiv3::Parameter::Header { parameter_data, .. } => {
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
            openapiv3::Parameter::Path { parameter_data, .. } => {
                path_params.insert(
                    parameter_data.name.clone(),
                    ValueSource::Property(PropertyId::from_path(&parameter_data.name)),
                );
                properties.push(MCPToolProperty {
                    name: parameter_data.name.clone(),
                    description: parameter_data.description.clone(),
                    required: parameter_data.required,
                    // TODO: don't hardcode string
                    type_: MCPToolPropertyType::String,
                });
            }
            openapiv3::Parameter::Cookie { .. } => todo!(),
        }
    }

    Ok(MCPTool {
        // TODO: allow multiple calls
        calls: vec![Call {
            method,
            path: file_path.to_string(),
            path_params,
            headers,
            query,
            body: None,
        }],
        properties,
        name: tool_name,
        description,
    })
}

fn resolve_parameter<'a>(
    openapi: &'a OpenAPI,
    param_ref: &'a ReferenceOr<Parameter>,
) -> Option<&'a Parameter> {
    match param_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let params = openapi.components.as_ref()?.parameters.get(ref_path)?;
            resolve_parameter(openapi, params)
        }
        ReferenceOr::Item(param) => Some(param),
    }
}

fn resolve_path<'a>(
    openapi: &'a OpenAPI,
    path_ref: &'a ReferenceOr<PathItem>,
) -> Option<&'a PathItem> {
    match path_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let path = openapi.paths.paths.get(ref_path)?;
            resolve_path(openapi, path)
        }
        ReferenceOr::Item(path) => Some(path),
    }
}
