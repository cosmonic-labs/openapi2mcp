use std::collections::{BTreeMap, HashMap, HashSet};

use convert_case::Casing;
use http::Method;
use openapiv3::{
    OAuth2Flows, OpenAPI, Parameter, PathItem, ReferenceOr, RequestBody, Schema, SecurityScheme,
};
use regex::Regex;

use crate::mcp_server::{
    Call, MCPServer, MCPTool, MCPToolProperty, MCPToolPropertyRequired, MCPToolPropertyType,
    PropertyId, Value, ValueSource,
};

pub const DEFAULT_MAX_TOOL_NAME_LENGTH: u32 = 80;

#[derive(Debug, Clone, Default)]
pub struct ConverterOptions {
    /// Regex patterns to include in the MCP server. If not provided, all tools will be included.
    pub include_tools: Option<Regex>,
    /// Methods to include in the MCP server. If not provided, all methods will be included.
    pub include_methods: Vec<http::Method>,
    /// Maximum length of the tool name. Default is `DEFAULT_MAX_TOOL_NAME_LENGTH`.
    pub max_tool_name_length: Option<u32>,
    /// Skip tool names that exceed the maximum length. Default is `false`.
    /// If true, the tool will be skipped and the next tool will be processed.
    /// If false, the tool throw an error.
    pub skip_long_tool_names: bool,
    /// OAuth2 information.
    pub oauth2_info: Option<openapiv3::AuthorizationCodeOAuth2Flow>,
}

pub fn openapi_to_mcp_server(
    openapi: OpenAPI,
    options: ConverterOptions,
) -> anyhow::Result<MCPServer> {
    let oauth2_info_from_spec =
        get_oauth2_info(&openapi).and_then(|info| info.authorization_code.as_ref());
    let oauth2_info_from_options = options.oauth2_info.clone();
    let oauth2_info = oauth2_info_from_options.or_else(|| oauth2_info_from_spec.cloned());

    let include_methods = &options.include_methods;
    let include_tools = &options.include_tools;

    let mut tools = Vec::new();
    for (path, path_item_ref) in &openapi.paths.paths {
        if matches!(&include_tools, Some(regex) if !regex.is_match(path)) {
            continue;
        }

        let path_item = resolve_path(&openapi, path_item_ref).unwrap();

        if let Some(operation) = &path_item.get {
            if !include_methods.is_empty() && !include_methods.contains(&Method::GET) {
                continue;
            }
            let tool = operation_to_tool(
                Method::GET,
                path,
                operation,
                &path_item.parameters,
                &openapi,
                &options,
            )?;
            if let Some(tool) = tool {
                tools.push(tool);
            }
            log::info!("Added GET tool for path: {}", path);
        }
        if let Some(operation) = &path_item.post {
            if !include_methods.is_empty() && !include_methods.contains(&Method::POST) {
                continue;
            }
            let tool = operation_to_tool(
                Method::POST,
                path,
                operation,
                &path_item.parameters,
                &openapi,
                &options,
            )?;
            if let Some(tool) = tool {
                tools.push(tool);
            }
            log::info!("Added POST tool for path: {}", path);
        }
        if let Some(operation) = &path_item.put {
            if !include_methods.is_empty() && !include_methods.contains(&Method::PUT) {
                continue;
            }
            let tool = operation_to_tool(
                Method::PUT,
                path,
                operation,
                &path_item.parameters,
                &openapi,
                &options,
            )?;
            if let Some(tool) = tool {
                tools.push(tool);
            }
            log::info!("Added PUT tool for path: {}", path);
        }
        if let Some(operation) = &path_item.delete {
            if !include_methods.is_empty() && !include_methods.contains(&Method::DELETE) {
                continue;
            }
            let tool = operation_to_tool(
                Method::DELETE,
                path,
                operation,
                &path_item.parameters,
                &openapi,
                &options,
            )?;
            if let Some(tool) = tool {
                tools.push(tool);
            }
            log::info!("Added DELETE tool for path: {}", path);
        }
        if let Some(operation) = &path_item.patch {
            if !include_methods.is_empty() && !include_methods.contains(&Method::PATCH) {
                continue;
            }
            let tool = operation_to_tool(
                Method::PATCH,
                path,
                operation,
                &path_item.parameters,
                &openapi,
                &options,
            )?;
            if let Some(tool) = tool {
                tools.push(tool);
            }
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
    options: &ConverterOptions,
) -> anyhow::Result<Option<MCPTool>> {
    let max_tool_name_length = options
        .max_tool_name_length
        .unwrap_or(DEFAULT_MAX_TOOL_NAME_LENGTH) as usize;
    let skip_long_tool_names = options.skip_long_tool_names;

    let tool_name = format!(
        "{}_{}",
        method.to_string().to_lowercase(),
        cleanup_string(path)
    );

    if tool_name.len() > max_tool_name_length {
        match skip_long_tool_names {
            true => return Ok(None),
            false => anyhow::bail!("Tool name {} exceeded the maximum length", tool_name),
        }
    }

    let description = operation
        .description
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    let mut path_params = BTreeMap::new();
    let mut query = BTreeMap::new();
    let mut headers = BTreeMap::new();
    let mut properties = Vec::new();
    let mut used_property_names = HashMap::new();
    let all_params = operation.parameters.iter().chain(route_params.iter());

    // TODO: take another look at the parameters
    for param_ref in all_params {
        let parameter = resolve_parameter(openapi, param_ref).unwrap();

        let parameter_data = match parameter {
            openapiv3::Parameter::Query { parameter_data, .. } => parameter_data,
            openapiv3::Parameter::Header { parameter_data, .. } => parameter_data,
            openapiv3::Parameter::Path { parameter_data, .. } => parameter_data,
            openapiv3::Parameter::Cookie { .. } => todo!(),
        };

        let required = if parameter_data.required {
            MCPToolPropertyRequired::Required
        } else {
            MCPToolPropertyRequired::Optional
        };
        // original name is the name of the parameter as it is in the OpenAPI spec
        let original_name = parameter_data.name.clone();

        // property name is the name of the property in the MCP server that might have a suffix if the name is already used
        let property_name = cleanup_string(&original_name);
        let property_name = match used_property_names.get(&property_name) {
            Some(count) => {
                let count = *count;
                used_property_names.insert(property_name.clone(), count + 1);
                format!("{}_{}", property_name, count + 1)
            }
            None => {
                used_property_names.insert(property_name.clone(), 1);
                property_name
            }
        };

        properties.push(MCPToolProperty {
            name: property_name.clone(),
            description: parameter_data.description.clone(),
            required,
            // TODO: don't hardcode string
            type_: MCPToolPropertyType::String,
        });

        match parameter {
            openapiv3::Parameter::Query { .. } => {
                query.insert(
                    original_name,
                    ValueSource::Property(PropertyId::from_query(&property_name)),
                );
            }
            openapiv3::Parameter::Header { .. } => {
                headers.insert(
                    original_name,
                    ValueSource::Property(PropertyId::from_header(&property_name)),
                );
            }
            openapiv3::Parameter::Path { .. } => {
                path_params.insert(
                    original_name,
                    ValueSource::Property(PropertyId::from_path(&property_name)),
                );
            }
            openapiv3::Parameter::Cookie { .. } => todo!(),
        };
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
                        let mut object = BTreeMap::new();
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
                log::error!("skipping schema_kind: {:#?}", a);
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

    Ok(Some(MCPTool {
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
    }))
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

fn cleanup_string(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if matches!(c, '-' | '/' | '\\' | ',' | '.') {
                return Some('_');
            }
            if c == '_' || c.is_alphanumeric() {
                return Some(c);
            }
            None
        })
        .collect::<String>()
        .to_case(convert_case::Case::Snake)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal OpenAPI spec for testing
    fn minimal_openapi() -> OpenAPI {
        OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a minimal operation for testing
    fn minimal_operation() -> openapiv3::Operation {
        openapiv3::Operation {
            description: Some("Test operation".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_short_tool_name_within_default_limit() {
        let openapi = minimal_openapi();
        let operation = minimal_operation();
        let options = ConverterOptions::default();

        let result = operation_to_tool(Method::GET, "/users", &operation, &[], &openapi, &options);

        assert!(result.is_ok());
        let tool = result.unwrap();
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "get_users");
    }

    #[test]
    fn test_long_tool_name_exceeds_limit_skip_false() {
        let openapi = minimal_openapi();
        let operation = minimal_operation();
        let options = ConverterOptions {
            max_tool_name_length: Some(10),
            skip_long_tool_names: false,
            ..Default::default()
        };

        // "get_users_profile" = 17 chars, exceeds limit of 10
        let result = operation_to_tool(
            Method::GET,
            "/users/profile",
            &operation,
            &[],
            &openapi,
            &options,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeded the maximum length"));
    }

    #[test]
    fn test_long_tool_name_exceeds_limit_skip_true() {
        let openapi = minimal_openapi();
        let operation = minimal_operation();
        let options = ConverterOptions {
            max_tool_name_length: Some(10),
            skip_long_tool_names: true,
            ..Default::default()
        };

        // "get_users_profile" = 17 chars, exceeds limit of 10
        let result = operation_to_tool(
            Method::GET,
            "/users/profile",
            &operation,
            &[],
            &openapi,
            &options,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Tool should be skipped
    }

    #[test]
    fn test_tool_name_at_exact_limit() {
        let openapi = minimal_openapi();
        let operation = minimal_operation();

        // "get_users" = 9 chars
        let options = ConverterOptions {
            max_tool_name_length: Some(9),
            skip_long_tool_names: false,
            ..Default::default()
        };

        let result = operation_to_tool(Method::GET, "/users", &operation, &[], &openapi, &options);

        assert!(result.is_ok());
        let tool = result.unwrap();
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "get_users");
    }

    #[test]
    fn test_tool_name_one_over_limit() {
        let openapi = minimal_openapi();
        let operation = minimal_operation();

        // "get_users" = 9 chars, limit is 8
        let options = ConverterOptions {
            max_tool_name_length: Some(8),
            skip_long_tool_names: false,
            ..Default::default()
        };

        let result = operation_to_tool(Method::GET, "/users", &operation, &[], &openapi, &options);

        assert!(result.is_err());
    }

    #[test]
    fn test_default_max_tool_name_length_constant() {
        assert_eq!(DEFAULT_MAX_TOOL_NAME_LENGTH, 80);
    }
}
