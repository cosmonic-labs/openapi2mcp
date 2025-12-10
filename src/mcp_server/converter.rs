use std::collections::{HashMap, HashSet};

use convert_case::Casing;
use http::Method;
use openapiv3::{
    OAuth2Flows, OpenAPI, Parameter, PathItem, ReferenceOr, RequestBody, Schema, SecurityScheme,
};

use crate::mcp_server::{
    Call, MCPServer, MCPTool, MCPToolProperty, MCPToolPropertyRequired, MCPToolPropertyType,
    PropertyId, Value, ValueSource,
};

pub fn openapi_to_mcp_server(openapi: OpenAPI) -> anyhow::Result<MCPServer> {
    let oauth2_info = get_oauth2_info(&openapi)
        .and_then(|info| info.authorization_code.as_ref())
        .cloned();

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
    anyhow::ensure!(openapi.servers.len() <= 1);
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
        oauth2_info,
    })
}

fn operation_to_tool(
    method: Method,
    path: &str,
    operation: &openapiv3::Operation,
    route_params: &[ReferenceOr<Parameter>],
    openapi: &OpenAPI,
) -> anyhow::Result<MCPTool> {
    let tool_name = format!(
        "{}_{}",
        method.to_string().to_lowercase(),
        path.trim_matches('/')
            .replace(',', "_")
            .replace('/', "_")
            .replace('-', "_")
            .replace('{', "")
            .replace('}', "")
            .to_case(convert_case::Case::Snake)
    );

    let description = operation
        .description
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    let mut path_params = HashMap::new();
    let mut query = HashMap::new();
    let mut headers = HashMap::new();
    let mut properties = Vec::new();
    let all_params = operation.parameters.iter().chain(route_params.iter());

    // TODO: take another look at the parameters
    for param_ref in all_params {
        let parameter_data = match resolve_parameter(openapi, param_ref).unwrap() {
            openapiv3::Parameter::Query { parameter_data, .. } => {
                query.insert(
                    parameter_data.name.clone(),
                    ValueSource::Property(PropertyId::from_query(&parameter_data.name)),
                );
                parameter_data
            }
            openapiv3::Parameter::Header { parameter_data, .. } => {
                headers.insert(
                    parameter_data.name.clone(),
                    ValueSource::Property(PropertyId::from_header(&parameter_data.name)),
                );
                parameter_data
            }
            openapiv3::Parameter::Path { parameter_data, .. } => {
                path_params.insert(
                    parameter_data.name.clone(),
                    ValueSource::Property(PropertyId::from_path(&parameter_data.name)),
                );
                parameter_data
            }
            openapiv3::Parameter::Cookie { .. } => todo!(),
        };
        let required = if parameter_data.required {
            MCPToolPropertyRequired::Required
        } else {
            MCPToolPropertyRequired::Optional
        };
        properties.push(MCPToolProperty {
            name: parameter_data.name.clone(),
            description: parameter_data.description.clone(),
            required,
            // TODO: don't hardcode string
            type_: MCPToolPropertyType::String,
        });
    }

    fn schema_kind_to_mcp_tool_property<'a>(
        schema_kind: &'a openapiv3::Schema,
        openapi: &'a OpenAPI,
    ) -> Option<MCPToolProperty> {
        match &schema_kind.schema_kind {
            openapiv3::SchemaKind::Type(type_) => {
                let type_ = match type_ {
                    openapiv3::Type::String(_string_type) => MCPToolPropertyType::String,
                    openapiv3::Type::Number(_number_type) => MCPToolPropertyType::Number,
                    openapiv3::Type::Integer(_integer_type) => {
                        // TODO: should be special type?
                        MCPToolPropertyType::Number
                    }
                    openapiv3::Type::Object(object_type) => {
                        let mut object = HashMap::new();
                        for (name, schema) in object_type.properties.iter() {
                            let schema = resolve_boxed_schema(openapi, schema).unwrap();
                            let value = schema_kind_to_mcp_tool_property(&schema, openapi);
                            if let Some(value) = value {
                                object.insert(name.clone(), value);
                            }
                        }
                        MCPToolPropertyType::Object(object)
                    }
                    openapiv3::Type::Array(array_type) => {
                        let schema =
                            resolve_boxed_schema(openapi, array_type.items.as_ref().unwrap())
                                .unwrap();
                        let value = schema_kind_to_mcp_tool_property(&schema, openapi).unwrap();
                        MCPToolPropertyType::Array(Box::new(value))
                    }
                    openapiv3::Type::Boolean(_boolean_type) => MCPToolPropertyType::Boolean,
                };

                let required_fields: HashSet<String> = match &schema_kind.schema_kind {
                    openapiv3::SchemaKind::Any(any_schema) => {
                        HashSet::from_iter(any_schema.required.clone())
                    }
                    _ => Default::default(),
                };

                let required = if let Some(default) = &schema_kind.schema_data.default {
                    MCPToolPropertyRequired::Default(default.clone())
                } else if required_fields
                    .contains(&schema_kind.schema_data.title.clone().unwrap_or_default())
                {
                    MCPToolPropertyRequired::Required
                } else {
                    MCPToolPropertyRequired::Optional
                };

                Some(MCPToolProperty {
                    name: schema_kind.schema_data.title.clone().unwrap_or_default(),
                    description: schema_kind.schema_data.description.clone(),
                    required,
                    type_,
                })
            }
            // openapiv3::SchemaKind::OneOf { one_of } => todo!(),
            // openapiv3::SchemaKind::AllOf { all_of } => todo!(),
            // openapiv3::SchemaKind::AllOf { all_of } => {
            //     let mut object = HashMap::new();
            //     for schema in all_of.iter() {
            //         let schema = resolve_schema(openapi, schema).unwrap();
            //         if let Some(value) = &schema_kind_to_mcp_tool_property_type(&schema.schema_kind, openapi) {
            //             object.insert(schema.schema_data.title.clone(), value);
            //         }
            //     }
            //     Some(MCPToolPropertyType::Object(object))
            // },
            // openapiv3::SchemaKind::AnyOf { any_of } => todo!(),
            // openapiv3::SchemaKind::Not { not } => todo!(),
            // openapiv3::SchemaKind::Any(any_schema) => todo!(),
            // _ => todo!(),
            a => {
                println!("schema_kind: {:#?}", a);
                None
            }
        }
    }

    let mut has_body = false;
    operation.request_body.as_ref().map(|body| {
        let body = resolve_request_body(openapi, &body).unwrap();

        // TODO: support non-json body
        if let Some(media_type) = &body.content.get("application/json") {
            headers.insert(
                "Content-Type".into(),
                ValueSource::Fixed(Value::String("application/json".into())),
            );
            let schema = match &media_type.schema {
                Some(schema) => resolve_schema(openapi, schema).unwrap(),
                None => todo!(),
            };

            let value = schema_kind_to_mcp_tool_property(&schema, openapi);
            if let Some(mut value) = value {
                value.name = "body".to_string();
                properties.push(value);
                has_body = true;
            }
        }
    });

    Ok(MCPTool {
        call: Call {
            method,
            path: path.to_string(),
            path_params,
            headers,
            query,
            body: has_body.then(|| ValueSource::Property(PropertyId::from_body("body"))),
        },
        properties,
        name: tool_name,
        description,
    })
}

fn get_oauth2_info(openapi: &OpenAPI) -> Option<&OAuth2Flows> {
    get_security_schemes(openapi)
        .iter()
        .find_map(|security_scheme| match security_scheme {
            SecurityScheme::OAuth2 { flows, .. } => Some(flows),
            _ => None,
        })
}

fn get_security_schemes(openapi: &OpenAPI) -> Vec<&SecurityScheme> {
    let components = openapi.components.as_ref();
    if let Some(components) = components {
        return components
            .security_schemes
            .iter()
            .filter_map(|(_, scheme_ref)| resolve_security_scheme(openapi, scheme_ref))
            .collect();
    }
    Vec::new()
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

fn resolve_request_body<'a>(
    openapi: &'a OpenAPI,
    request_body_ref: &'a ReferenceOr<RequestBody>,
) -> Option<&'a RequestBody> {
    match request_body_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let path = openapi.components.as_ref()?.request_bodies.get(ref_path)?;
            resolve_request_body(openapi, path)
        }
        ReferenceOr::Item(request_body) => Some(request_body),
    }
}

fn resolve_schema<'a>(
    openapi: &'a OpenAPI,
    schema_ref: &'a ReferenceOr<Schema>,
) -> Option<&'a Schema> {
    match schema_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let path = openapi.components.as_ref()?.schemas.get(ref_path)?;
            resolve_schema(openapi, path)
        }
        ReferenceOr::Item(schema) => Some(schema),
    }
}

fn resolve_security_scheme<'a>(
    openapi: &'a OpenAPI,
    schema_ref: &'a ReferenceOr<SecurityScheme>,
) -> Option<&'a SecurityScheme> {
    match schema_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let path = openapi
                .components
                .as_ref()?
                .security_schemes
                .get(ref_path)?;
            resolve_security_scheme(openapi, path)
        }
        ReferenceOr::Item(schema) => Some(schema),
    }
}

fn resolve_boxed_schema<'a>(
    openapi: &'a OpenAPI,
    schema_ref: &'a ReferenceOr<Box<Schema>>,
) -> Option<&'a Schema> {
    match schema_ref {
        ReferenceOr::Reference { reference } => {
            let ref_path = reference.split("/").last().unwrap();
            let path = openapi.components.as_ref()?.schemas.get(ref_path)?;
            resolve_schema(openapi, path)
        }
        ReferenceOr::Item(schema) => Some(schema),
    }
}
