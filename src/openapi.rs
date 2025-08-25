use openapiv3::{OpenAPI, ReferenceOr, SchemaData, SchemaKind, Type};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Re-export openapiv3 types for easier access
pub use openapiv3::{
    Components, Info, MediaType, Operation, Parameter, PathItem, RequestBody, Response, Schema,
};

// Wrapper type to add our custom methods
#[derive(Debug, Clone)]
pub struct OpenApiSpec {
    pub inner: OpenAPI,
}

impl OpenApiSpec {
    pub fn new(inner: OpenAPI) -> Self {
        Self { inner }
    }

    // Delegate common field access
    pub fn openapi(&self) -> &str {
        &self.inner.openapi
    }

    pub fn info(&self) -> &openapiv3::Info {
        &self.inner.info
    }

    pub fn paths(&self) -> &openapiv3::Paths {
        &self.inner.paths
    }

    pub fn components(&self) -> &Option<openapiv3::Components> {
        &self.inner.components
    }
}

pub fn parse_openapi_spec_from_path<P: AsRef<Path>>(path: P) -> crate::Result<OpenApiSpec> {
    let content = fs::read_to_string(&path)?;

    let inner: OpenAPI = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&content)
            .map_err(|e| crate::Error::Parse(format!("Failed to parse JSON: {}", e)))?
    } else {
        serde_yaml::from_str(&content)
            .map_err(|e| crate::Error::Parse(format!("Failed to parse YAML: {}", e)))?
    };

    let spec = OpenApiSpec::new(inner);
    validate_spec(&spec)?;
    Ok(spec)
}

pub fn parse_openapi_spec(spec: impl AsRef<str>) -> crate::Result<OpenApiSpec> {
    let content = spec.as_ref();
    // TODO: Should be ok without parsing json right? could fallback
    let inner: OpenAPI = serde_yaml::from_str(content)
        .map_err(|e| crate::Error::Parse(format!("Failed to parse YAML: {}", e)))?;

    let spec = OpenApiSpec::new(inner);
    validate_spec(&spec)?;
    Ok(spec)
}

impl OpenApiSpec {
    /// Resolve a ReferenceOr to get a simple schema representation for MCP generation
    /// This is a simplified version for Phase 1 - will be enhanced in future phases
    pub fn resolve_schema_simple(
        &self,
        _schema_ref: &ReferenceOr<Schema>,
    ) -> crate::Result<ResolvedSchema> {
        // For Phase 1, return a simple placeholder schema
        // TODO: Implement proper reference resolution in future phases
        Ok(ResolvedSchema::Simple {
            schema_type: "string".to_string(),
            format: None,
            additional_properties: HashMap::new(),
        })
    }

    /// Convert openapiv3 schema to a resolved schema for MCP generation
    pub fn resolve_schema(
        &self,
        schema_ref: &ReferenceOr<Schema>,
    ) -> crate::Result<ResolvedSchema> {
        // For Phase 1, use simplified resolution - will be enhanced in future phases
        match schema_ref {
            ReferenceOr::Item(schema) => self.resolve_schema_direct(schema),
            ReferenceOr::Reference { .. } => {
                // TODO: Implement proper reference resolution in future phases
                Ok(ResolvedSchema::Simple {
                    schema_type: "string".to_string(),
                    format: None,
                    additional_properties: HashMap::new(),
                })
            }
        }
    }

    /// Resolve a direct schema (not a reference) to ResolvedSchema
    fn resolve_schema_direct(&self, schema: &Schema) -> crate::Result<ResolvedSchema> {
        match &schema.schema_kind {
            SchemaKind::Type(Type::String(_string_type)) => {
                Ok(ResolvedSchema::Simple {
                    schema_type: "string".to_string(),
                    format: None, // TODO: Properly handle VariantOrUnknownOrEmpty format
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::Type(Type::Number(_number_type)) => {
                Ok(ResolvedSchema::Simple {
                    schema_type: "number".to_string(),
                    format: None, // TODO: Properly handle VariantOrUnknownOrEmpty format
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::Type(Type::Integer(_integer_type)) => {
                Ok(ResolvedSchema::Simple {
                    schema_type: "integer".to_string(),
                    format: None, // TODO: Properly handle VariantOrUnknownOrEmpty format
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::Type(Type::Boolean(_)) => Ok(ResolvedSchema::Simple {
                schema_type: "boolean".to_string(),
                format: None,
                additional_properties: self.extract_additional_properties(&schema.schema_data),
            }),
            SchemaKind::Type(Type::Array(_array_type)) => {
                // TODO: Properly handle array items in future phases
                Ok(ResolvedSchema::Array {
                    schema_type: "array".to_string(),
                    items: None,
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::Type(Type::Object(object_type)) => {
                // For now, create a simplified object schema with basic property info
                // This allows us to extract parameter names for the integration tests
                let mut resolved_properties = None;

                // Extract basic property names and types
                if !object_type.properties.is_empty() {
                    let mut prop_map = std::collections::HashMap::new();
                    for (prop_name, _prop_schema_ref) in &object_type.properties {
                        // Create a simple string property for each - can be enhanced later
                        let simple_prop = ResolvedSchema::Simple {
                            schema_type: "string".to_string(),
                            format: None,
                            additional_properties: std::collections::HashMap::new(),
                        };
                        prop_map.insert(prop_name.clone(), Box::new(simple_prop));
                    }
                    resolved_properties = Some(prop_map);
                }

                // Get required properties if they exist
                let required_from_object = if !object_type.required.is_empty() {
                    Some(object_type.required.clone())
                } else {
                    None
                };

                Ok(ResolvedSchema::Object {
                    schema_type: Some("object".to_string()),
                    properties: resolved_properties,
                    required: required_from_object,
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::OneOf { one_of } => {
                // For now, treat oneOf as the first schema - could be enhanced later
                if let Some(first_schema) = one_of.first() {
                    self.resolve_schema(first_schema)
                } else {
                    Ok(ResolvedSchema::Simple {
                        schema_type: "object".to_string(),
                        format: None,
                        additional_properties: HashMap::new(),
                    })
                }
            }
            SchemaKind::AllOf { all_of } => {
                // For now, merge all schemas into one object - could be enhanced later
                let mut merged_properties = HashMap::new();
                let mut merged_required = Vec::new();

                for schema_ref in all_of {
                    let resolved = self.resolve_schema(schema_ref)?;
                    if let ResolvedSchema::Object {
                        properties,
                        required,
                        ..
                    } = resolved
                    {
                        if let Some(props) = properties {
                            merged_properties.extend(props);
                        }
                        if let Some(req) = required {
                            merged_required.extend(req);
                        }
                    }
                }

                Ok(ResolvedSchema::Object {
                    schema_type: Some("object".to_string()),
                    properties: if merged_properties.is_empty() {
                        None
                    } else {
                        Some(merged_properties)
                    },
                    required: if merged_required.is_empty() {
                        None
                    } else {
                        Some(merged_required)
                    },
                    additional_properties: self.extract_additional_properties(&schema.schema_data),
                })
            }
            SchemaKind::AnyOf { any_of } => {
                // For now, treat anyOf as the first schema - could be enhanced later
                if let Some(first_schema) = any_of.first() {
                    self.resolve_schema(first_schema)
                } else {
                    Ok(ResolvedSchema::Simple {
                        schema_type: "object".to_string(),
                        format: None,
                        additional_properties: HashMap::new(),
                    })
                }
            }
            SchemaKind::Not { .. } => {
                // Not schemas are complex - for now just return a generic object
                Ok(ResolvedSchema::Simple {
                    schema_type: "object".to_string(),
                    format: None,
                    additional_properties: HashMap::new(),
                })
            }
            SchemaKind::Any(_) => {
                // Any schema - return generic object
                Ok(ResolvedSchema::Simple {
                    schema_type: "object".to_string(),
                    format: None,
                    additional_properties: HashMap::new(),
                })
            }
        }
    }

    /// Extract additional properties from SchemaData for backward compatibility
    fn extract_additional_properties(
        &self,
        schema_data: &SchemaData,
    ) -> HashMap<String, serde_json::Value> {
        let mut additional = HashMap::new();

        if let Some(title) = &schema_data.title {
            additional.insert(
                "title".to_string(),
                serde_json::Value::String(title.clone()),
            );
        }
        if let Some(description) = &schema_data.description {
            additional.insert(
                "description".to_string(),
                serde_json::Value::String(description.clone()),
            );
        }
        if let Some(default) = &schema_data.default {
            additional.insert("default".to_string(), default.clone());
        }
        if let Some(example) = &schema_data.example {
            additional.insert("example".to_string(), example.clone());
        }

        additional
    }
}

/// A schema with all references resolved
#[derive(Debug, Clone)]
pub enum ResolvedSchema {
    Object {
        schema_type: Option<String>,
        properties: Option<HashMap<String, Box<ResolvedSchema>>>,
        required: Option<Vec<String>>,
        additional_properties: HashMap<String, serde_json::Value>,
    },
    Array {
        schema_type: String,
        items: Option<Box<ResolvedSchema>>,
        additional_properties: HashMap<String, serde_json::Value>,
    },
    Simple {
        schema_type: String,
        format: Option<String>,
        additional_properties: HashMap<String, serde_json::Value>,
    },
}

fn validate_spec(spec: &OpenApiSpec) -> crate::Result<()> {
    if !spec.openapi().starts_with("3.") {
        return Err(crate::Error::Validation(
            "Only OpenAPI 3.x specifications are supported".to_string(),
        ));
    }

    if spec.info().title.is_empty() {
        return Err(crate::Error::Validation(
            "API title is required".to_string(),
        ));
    }

    // Additional validation using openapiv3 structure
    if spec.paths().paths.is_empty() {
        return Err(crate::Error::Validation(
            "OpenAPI spec must have at least one path".to_string(),
        ));
    }

    // Validate that all references can be resolved
    for (_, path_item_ref) in &spec.paths().paths {
        if let ReferenceOr::Reference { reference } = path_item_ref {
            return Err(crate::Error::Validation(format!(
                "Path item references are not supported: {}",
                reference
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_spec_from_json() -> OpenApiSpec {
        let spec_json = r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0",
    "description": "A test API"
  },
  "paths": {
    "/users": {
      "get": {
        "summary": "Get users",
        "description": "Retrieve a list of users",
        "operationId": "getUsers",
        "parameters": [
          {
            "name": "limit",
            "in": "query",
            "required": false,
            "schema": {
              "type": "integer"
            },
            "description": "Number of items to return"
          }
        ],
        "responses": {
          "200": {
            "description": "Successful response"
          }
        }
      },
      "post": {
        "summary": "Create user",
        "operationId": "createUser",
        "requestBody": {
          "description": "User data",
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "name": {
                    "type": "string"
                  }
                },
                "required": ["name"]
              }
            }
          }
        },
        "responses": {
          "201": {
            "description": "User created"
          }
        }
      }
    }
  }
}"#;

        let inner: openapiv3::OpenAPI = serde_json::from_str(spec_json).unwrap();
        OpenApiSpec::new(inner)
    }

    fn create_test_spec_with_components() -> OpenApiSpec {
        let spec_json = r##"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/user": {
      "get": {
        "summary": "Get user",
        "operationId": "getUser",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/User"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": {
            "type": "integer"
          },
          "name": {
            "type": "string"
          }
        },
        "required": ["id", "name"]
      }
    }
  }
}"##;

        let inner: openapiv3::OpenAPI = serde_json::from_str(spec_json).unwrap();
        OpenApiSpec::new(inner)
    }

    fn create_invalid_spec() -> OpenApiSpec {
        let spec_json = r#"{
  "openapi": "2.0.0",
  "info": {
    "title": "",
    "version": "1.0.0"
  },
  "paths": {}
}"#;

        let inner: openapiv3::OpenAPI = serde_json::from_str(spec_json).unwrap();
        OpenApiSpec::new(inner)
    }

    #[test]
    fn test_parse_valid_yaml_spec() {
        let spec_yaml = r#"
openapi: "3.0.0"
info:
  title: "Test API"
  version: "1.0.0"
  description: "A test API"
paths:
  /users:
    get:
      summary: "Get users"
      description: "Retrieve a list of users"
      operationId: "getUsers"
      responses:
        "200":
          description: "Successful response"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", spec_yaml).unwrap();

        let result = parse_openapi_spec_from_path(temp_file.path());
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.openapi(), "3.0.0");
        assert_eq!(spec.info().title, "Test API");
        assert_eq!(spec.info().version, "1.0.0");
        assert!(spec.paths().paths.contains_key("/users"));
    }

    #[test]
    fn test_parse_valid_json_spec() {
        let spec_json = r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0",
    "description": "A test API"
  },
  "paths": {
    "/users": {
      "get": {
        "summary": "Get users",
        "description": "Retrieve a list of users",
        "operationId": "getUsers",
        "responses": {
          "200": {
            "description": "Successful response"
          }
        }
      }
    }
  }
}"#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{}", spec_json).unwrap();

        let result = parse_openapi_spec_from_path(temp_file.path());
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.openapi(), "3.0.0");
        assert_eq!(spec.info().title, "Test API");
    }

    #[test]
    fn test_parse_invalid_spec() {
        let invalid_yaml = "invalid: yaml: content:";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_yaml).unwrap();

        let result = parse_openapi_spec_from_path(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::Parse(_)));
    }

    #[test]
    fn test_validate_spec_invalid_version() {
        let spec = create_invalid_spec();
        let result = validate_spec(&spec);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::Validation(_)));
    }

    #[test]
    fn test_validate_spec_empty_title() {
        let spec = create_invalid_spec();
        let result = validate_spec(&spec);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::Validation(_)));
    }

    #[test]
    fn test_validate_spec_valid() {
        let spec = create_test_spec_from_json();
        let result = validate_spec(&spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_file() {
        let result = parse_openapi_spec_from_path("/nonexistent/file.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::Io(_)));
    }

    #[test]
    fn test_resolve_schema_simple() {
        let spec = create_test_spec_from_json();

        // Test that we can create a simple resolved schema (placeholder for Phase 1)
        let result = spec.resolve_schema_simple(&openapiv3::ReferenceOr::Item(openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(Default::default())),
        }));

        assert!(result.is_ok());
        match result.unwrap() {
            ResolvedSchema::Simple { schema_type, .. } => {
                assert_eq!(schema_type, "string");
            }
            _ => panic!("Expected Simple schema"),
        }
    }

    #[test]
    fn test_resolve_schema_reference_placeholder() {
        let spec = create_test_spec_with_components();

        // Test that reference resolution returns placeholder for Phase 1
        let reference = openapiv3::ReferenceOr::Reference {
            reference: "#/components/schemas/User".to_string(),
        };

        let result = spec.resolve_schema(&reference);
        assert!(result.is_ok());

        // In Phase 1, references resolve to simple placeholders
        match result.unwrap() {
            ResolvedSchema::Simple { schema_type, .. } => {
                assert_eq!(schema_type, "string");
            }
            _ => panic!("Expected placeholder Simple schema"),
        }
    }
}
