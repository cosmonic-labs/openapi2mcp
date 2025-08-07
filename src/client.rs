use crate::openapi::{OpenApiSpec, Operation};
use openapiv3::ReferenceOr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub operation_id: String,
    pub description: String,
    pub parameters: Vec<ApiParameter>,
    pub request_body: Option<ApiRequestBody>,
    pub responses: HashMap<String, ApiResponse>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiParameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub schema_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone)]
pub struct ApiRequestBody {
    pub required: bool,
    pub content_types: Vec<String>,
    pub schema_type: String,
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status_code: String,
    pub description: String,
    pub content_types: Vec<String>,
    pub schema_type: Option<String>,
}

#[derive(Debug)]
pub struct ApiClient {
    pub spec: OpenApiSpec,
    pub endpoints: Vec<ApiEndpoint>,
}

impl ApiClient {
    pub fn new(spec: OpenApiSpec) -> crate::Result<Self> {
        log::info!("Creating API client for spec: {}", spec.info().title);
        let mut client = ApiClient {
            spec,
            endpoints: Vec::new(),
        };
        
        client.extract_endpoints()?;
        log::info!("Extracted {} API endpoints", client.endpoints.len());
        Ok(client)
    }

    fn extract_endpoints(&mut self) -> crate::Result<()> {
        let base_url = self.extract_base_url();
        log::debug!("Using base URL: {:?}", base_url);

        for (path, path_item_ref) in &self.spec.paths().paths {
            let path_item = match path_item_ref {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { reference } => {
                    log::warn!("Skipping path item reference: {}", reference);
                    continue;
                }
            };

            if let Some(operation) = &path_item.get {
                self.endpoints.push(self.create_endpoint("GET", path, operation, &base_url)?);
            }
            if let Some(operation) = &path_item.post {
                self.endpoints.push(self.create_endpoint("POST", path, operation, &base_url)?);
            }
            if let Some(operation) = &path_item.put {
                self.endpoints.push(self.create_endpoint("PUT", path, operation, &base_url)?);
            }
            if let Some(operation) = &path_item.delete {
                self.endpoints.push(self.create_endpoint("DELETE", path, operation, &base_url)?);
            }
            if let Some(operation) = &path_item.patch {
                self.endpoints.push(self.create_endpoint("PATCH", path, operation, &base_url)?);
            }
        }

        Ok(())
    }

    fn extract_base_url(&self) -> Option<String> {
        self.spec.inner.servers.first().and_then(|server| {
            Some(server.url.clone())
        })
    }

    fn create_endpoint(
        &self,
        method: &str,
        path: &str,
        operation: &Operation,
        base_url: &Option<String>,
    ) -> crate::Result<ApiEndpoint> {
        let operation_id = operation.operation_id.clone()
            .unwrap_or_else(|| format!("{}_{}", method.to_lowercase(), 
                path.replace('/', "_").replace('{', "").replace('}', "")));

        let description = operation.summary.clone()
            .or_else(|| operation.description.clone())
            .unwrap_or_else(|| format!("{} {}", method, path));

        let parameters = self.extract_parameters(&operation.parameters)?;
        let request_body = self.extract_request_body(&operation.request_body)?;
        let responses = self.extract_responses(&operation.responses)?;

        Ok(ApiEndpoint {
            method: method.to_string(),
            path: path.to_string(),
            operation_id,
            description,
            parameters,
            request_body,
            responses,
            base_url: base_url.clone(),
        })
    }

    fn extract_parameters(&self, params: &[ReferenceOr<openapiv3::Parameter>]) -> crate::Result<Vec<ApiParameter>> {
        let mut parameters = Vec::new();

        for param_ref in params {
            let param = match param_ref {
                ReferenceOr::Item(p) => p,
                ReferenceOr::Reference { reference } => {
                    log::warn!("Skipping parameter reference: {}", reference);
                    continue;
                }
            };

            let _location = match param {
                openapiv3::Parameter::Query { parameter_data, .. } => {
                    parameters.push(ApiParameter {
                        name: parameter_data.name.clone(),
                        location: ParameterLocation::Query,
                        required: parameter_data.required,
                        schema_type: self.extract_parameter_type(param)?,
                        description: parameter_data.description.clone(),
                    });
                }
                openapiv3::Parameter::Header { parameter_data, .. } => {
                    parameters.push(ApiParameter {
                        name: parameter_data.name.clone(),
                        location: ParameterLocation::Header,
                        required: parameter_data.required,
                        schema_type: self.extract_parameter_type(param)?,
                        description: parameter_data.description.clone(),
                    });
                }
                openapiv3::Parameter::Path { parameter_data, .. } => {
                    parameters.push(ApiParameter {
                        name: parameter_data.name.clone(),
                        location: ParameterLocation::Path,
                        required: parameter_data.required,
                        schema_type: self.extract_parameter_type(param)?,
                        description: parameter_data.description.clone(),
                    });
                }
                openapiv3::Parameter::Cookie { parameter_data, .. } => {
                    parameters.push(ApiParameter {
                        name: parameter_data.name.clone(),
                        location: ParameterLocation::Cookie,
                        required: parameter_data.required,
                        schema_type: self.extract_parameter_type(param)?,
                        description: parameter_data.description.clone(),
                    });
                }
            };
        }

        Ok(parameters)
    }

    fn extract_parameter_type(&self, _param: &openapiv3::Parameter) -> crate::Result<String> {
        // For now, return string as default type since the openapiv3 parameter structure
        // doesn't expose the schema in the way we expected. This could be enhanced later
        // by investigating the actual structure of the openapiv3::Parameter enum
        Ok("string".to_string())
    }

    fn extract_request_body(&self, request_body_ref: &Option<ReferenceOr<openapiv3::RequestBody>>) -> crate::Result<Option<ApiRequestBody>> {
        let request_body_ref = match request_body_ref {
            Some(rb) => rb,
            None => return Ok(None),
        };

        let request_body = match request_body_ref {
            ReferenceOr::Item(rb) => rb,
            ReferenceOr::Reference { reference } => {
                log::warn!("Skipping request body reference: {}", reference);
                return Ok(None);
            }
        };

        let content_types: Vec<String> = request_body.content.keys().cloned().collect();
        
        // Get the first content type's schema for simplification
        let schema_type = if let Some((_, media_type)) = request_body.content.iter().next() {
            if media_type.schema.is_some() {
                "object".to_string() // Simplified - could be enhanced
            } else {
                "any".to_string()
            }
        } else {
            "any".to_string()
        };

        Ok(Some(ApiRequestBody {
            required: request_body.required,
            content_types,
            schema_type,
        }))
    }

    fn extract_responses(&self, responses: &openapiv3::Responses) -> crate::Result<HashMap<String, ApiResponse>> {
        let mut response_map = HashMap::new();

        // Handle default response
        if let Some(default_response_ref) = &responses.default {
            let response = self.extract_single_response("default", default_response_ref)?;
            response_map.insert("default".to_string(), response);
        }

        // Handle status-code specific responses
        for (status_code, response_ref) in &responses.responses {
            let response = self.extract_single_response(&status_code.to_string(), response_ref)?;
            response_map.insert(status_code.to_string(), response);
        }

        Ok(response_map)
    }

    fn extract_single_response(&self, status_code: &str, response_ref: &ReferenceOr<openapiv3::Response>) -> crate::Result<ApiResponse> {
        let response = match response_ref {
            ReferenceOr::Item(resp) => resp,
            ReferenceOr::Reference { reference } => {
                log::warn!("Using placeholder for response reference: {}", reference);
                return Ok(ApiResponse {
                    status_code: status_code.to_string(),
                    description: format!("Response reference: {}", reference),
                    content_types: vec![],
                    schema_type: None,
                });
            }
        };

        let content_types: Vec<String> = response.content.keys().cloned().collect();
        
        let schema_type = if let Some((_, media_type)) = response.content.iter().next() {
            if media_type.schema.is_some() {
                Some("object".to_string()) // Simplified
            } else {
                None
            }
        } else {
            None
        };

        Ok(ApiResponse {
            status_code: status_code.to_string(),
            description: response.description.clone(),
            content_types,
            schema_type,
        })
    }

    pub fn generate_typescript_client(&self) -> crate::Result<String> {
        log::info!("Generating TypeScript API client");
        let mut code = String::new();

        code.push_str(&format!(
            r#"// Generated API client for {}
// This file contains HTTP client code for consuming the API endpoints

export interface ApiClientConfig {{
  baseUrl?: string;
  timeout?: number;
  headers?: Record<string, string>;
}}

export class ApiClient {{
  private baseUrl: string;
  private timeout: number;
  private defaultHeaders: Record<string, string>;

  constructor(config: ApiClientConfig = {{}}) {{
    this.baseUrl = config.baseUrl || '{}';
    this.timeout = config.timeout || 30000;
    this.defaultHeaders = config.headers || {{}};
  }}

  private async makeRequest<T>(
    method: string,
    path: string,
    options: {{
      params?: Record<string, any>;
      body?: any;
      headers?: Record<string, string>;
    }} = {{}}
  ): Promise<T> {{
    const url = new URL(path, this.baseUrl);
    
    // Add query parameters
    if (options.params) {{
      Object.entries(options.params).forEach(([key, value]) => {{
        if (value !== undefined && value !== null) {{
          url.searchParams.append(key, String(value));
        }}
      }});
    }}

    const headers = {{
      'Content-Type': 'application/json',
      ...this.defaultHeaders,
      ...options.headers,
    }};

    const fetchOptions: RequestInit = {{
      method,
      headers,
    }};

    if (options.body && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {{
      fetchOptions.body = JSON.stringify(options.body);
    }}

    try {{
      const response = await fetch(url.toString(), fetchOptions);
      
      if (!response.ok) {{
        throw new Error(`HTTP error! status: ${{response.status}}`);
      }}

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {{
        return await response.json();
      }} else {{
        return await response.text() as unknown as T;
      }}
    }} catch (error) {{
      console.error('API request failed:', error);
      throw error;
    }}
  }}

"#,
            self.spec.info().title,
            self.endpoints.first().and_then(|e| e.base_url.as_ref()).unwrap_or(&"https://api.example.com".to_string())
        ));

        // Generate methods for each endpoint
        for endpoint in &self.endpoints {
            code.push_str(&self.generate_typescript_method(endpoint)?);
        }

        code.push_str("}\n\n");
        code.push_str(&self.generate_typescript_interfaces()?);

        Ok(code)
    }

    fn generate_typescript_method(&self, endpoint: &ApiEndpoint) -> crate::Result<String> {
        let mut code = String::new();
        let method_name = &endpoint.operation_id;
        
        // Build parameter list
        let mut param_parts = Vec::new();
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();

        for param in &endpoint.parameters {
            match param.location {
                ParameterLocation::Path => {
                    param_parts.push(format!("{}: {}", param.name, self.ts_type(&param.schema_type)));
                    path_params.push(param.name.clone());
                }
                ParameterLocation::Query => {
                    if param.required {
                        param_parts.push(format!("{}: {}", param.name, self.ts_type(&param.schema_type)));
                    } else {
                        param_parts.push(format!("{}?: {}", param.name, self.ts_type(&param.schema_type)));
                    }
                    query_params.push(param.name.clone());
                }
                ParameterLocation::Header => {
                    if param.required {
                        param_parts.push(format!("{}: {}", param.name, self.ts_type(&param.schema_type)));
                    } else {
                        param_parts.push(format!("{}?: {}", param.name, self.ts_type(&param.schema_type)));
                    }
                    header_params.push(param.name.clone());
                }
                ParameterLocation::Cookie => {
                    // Skip cookie parameters for now
                }
            }
        }

        if let Some(body) = &endpoint.request_body {
            if body.required {
                param_parts.push("body: any".to_string());
            } else {
                param_parts.push("body?: any".to_string());
            }
        }

        let params_str = if param_parts.is_empty() {
            String::new()
        } else {
            format!("{}", param_parts.join(", "))
        };

        // Determine return type
        let return_type = if let Some(response) = endpoint.responses.get("200")
            .or_else(|| endpoint.responses.get("201"))
            .or_else(|| endpoint.responses.get("default"))
        {
            if response.schema_type.is_some() {
                "any".to_string()
            } else {
                "void".to_string()
            }
        } else {
            "any".to_string()
        };

        code.push_str(&format!(
            r#"  /**
   * {}
   */
  async {}({}): Promise<{}> {{
"#,
            endpoint.description,
            method_name,
            params_str,
            return_type
        ));

        // Build path with substitutions
        let mut api_path = endpoint.path.clone();
        for path_param in &path_params {
            api_path = api_path.replace(&format!("{{{}}}", path_param), &format!("${{{}}}", path_param));
        }

        // Build query parameters object
        if !query_params.is_empty() {
            code.push_str("    const params: Record<string, any> = {};\n");
            for param in &query_params {
                code.push_str(&format!("    if ({} !== undefined) params['{}'] = {};\n", param, param, param));
            }
        }

        // Build headers object
        if !header_params.is_empty() {
            code.push_str("    const headers: Record<string, string> = {};\n");
            for param in &header_params {
                code.push_str(&format!("    if ({} !== undefined) headers['{}'] = String({});\n", param, param, param));
            }
        }

        // Make the request
        code.push_str(&format!(
            "    return this.makeRequest<{}>('{}', `{}`, {{\n",
            return_type,
            endpoint.method,
            api_path
        ));

        if !query_params.is_empty() {
            code.push_str("      params,\n");
        }
        if !header_params.is_empty() {
            code.push_str("      headers,\n");
        }
        if endpoint.request_body.is_some() {
            code.push_str("      body,\n");
        }

        code.push_str("    });\n");
        code.push_str("  }\n\n");

        Ok(code)
    }

    fn generate_typescript_interfaces(&self) -> crate::Result<String> {
        // For now, return empty interfaces - could be enhanced with proper schema generation
        Ok("// TODO: Add TypeScript interfaces for request/response types\n".to_string())
    }

    fn ts_type(&self, schema_type: &str) -> &str {
        match schema_type {
            "integer" => "number",
            "number" => "number",
            "boolean" => "boolean",
            "array" => "any[]",
            "object" => "any",
            _ => "string",
        }
    }

    pub fn generate_rust_client(&self) -> crate::Result<String> {
        log::info!("Generating Rust API client");
        let mut code = String::new();

        code.push_str(&format!(
            r#"// Generated API client for {}
// This file contains HTTP client code for consuming the API endpoints

use reqwest::{{Client, Response}};
use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ApiClientConfig {{
    pub base_url: String,
    pub timeout: std::time::Duration,
    pub default_headers: HashMap<String, String>,
}}

impl Default for ApiClientConfig {{
    fn default() -> Self {{
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        Self {{
            base_url: "{}".to_string(),
            timeout: std::time::Duration::from_secs(30),
            default_headers: headers,
        }}
    }}
}}

#[derive(Debug)]
pub struct ApiClient {{
    client: Client,
    config: ApiClientConfig,
}}

impl ApiClient {{
    pub fn new(config: ApiClientConfig) -> Result<Self> {{
        let client = Client::builder()
            .timeout(config.timeout)
            .build()?;

        Ok(Self {{
            client,
            config,
        }})
    }}

    pub fn with_default_config() -> Result<Self> {{
        Self::new(ApiClientConfig::default())
    }}

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&HashMap<String, String>>,
        body: Option<&impl Serialize>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<T> {{
        let mut url = url::Url::parse(&self.config.base_url)?;
        url.set_path(path);

        if let Some(params) = params {{
            for (key, value) in params {{
                url.query_pairs_mut().append_pair(key, value);
            }}
        }}

        let mut request = self.client.request(method, url);

        // Add default headers
        for (key, value) in &self.config.default_headers {{
            request = request.header(key, value);
        }}

        // Add custom headers
        if let Some(headers) = headers {{
            for (key, value) in headers {{
                request = request.header(key, value);
            }}
        }}

        // Add body if provided
        if let Some(body) = body {{
            request = request.json(body);
        }}

        let response = request.send().await?;
        
        if !response.status().is_success() {{
            return Err(anyhow::anyhow!("HTTP error: {{}}", response.status()));
        }}

        let result = response.json::<T>().await?;
        Ok(result)
    }}

"#,
            self.spec.info().title,
            self.endpoints.first().and_then(|e| e.base_url.as_ref()).unwrap_or(&"https://api.example.com".to_string())
        ));

        // Generate methods for each endpoint
        for endpoint in &self.endpoints {
            code.push_str(&self.generate_rust_method(endpoint)?);
        }

        code.push_str("}\n\n");
        code.push_str(&self.generate_rust_types()?);

        Ok(code)
    }

    fn generate_rust_method(&self, endpoint: &ApiEndpoint) -> crate::Result<String> {
        let mut code = String::new();
        let method_name = &endpoint.operation_id;
        
        // Build parameter list
        let mut param_parts = Vec::new();
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();

        for param in &endpoint.parameters {
            let rust_type = self.rust_type(&param.schema_type);
            match param.location {
                ParameterLocation::Path => {
                    param_parts.push(format!("{}: {}", param.name, rust_type));
                    path_params.push(param.name.clone());
                }
                ParameterLocation::Query => {
                    if param.required {
                        param_parts.push(format!("{}: {}", param.name, rust_type));
                    } else {
                        param_parts.push(format!("{}: Option<{}>", param.name, rust_type));
                    }
                    query_params.push(param.name.clone());
                }
                ParameterLocation::Header => {
                    if param.required {
                        param_parts.push(format!("{}: {}", param.name, rust_type));
                    } else {
                        param_parts.push(format!("{}: Option<{}>", param.name, rust_type));
                    }
                    header_params.push(param.name.clone());
                }
                ParameterLocation::Cookie => {
                    // Skip cookie parameters for now
                }
            }
        }

        if let Some(body) = &endpoint.request_body {
            if body.required {
                param_parts.push("body: &impl Serialize".to_string());
            } else {
                param_parts.push("body: Option<&impl Serialize>".to_string());
            }
        }

        let params_str = if param_parts.is_empty() {
            "&self".to_string()
        } else {
            format!("&self, {}", param_parts.join(", "))
        };

        code.push_str(&format!(
            r#"    /// {}
    pub async fn {}({}) -> Result<serde_json::Value> {{
"#,
            endpoint.description,
            method_name,
            params_str
        ));

        // Build path with substitutions
        let mut api_path = endpoint.path.clone();
        for path_param in &path_params {
            // Replace {param} with ${param} for string interpolation
            api_path = api_path.replace(&format!("{{{}}}", path_param), &format!("${{{}}}", path_param));
        }

        // Build query parameters
        if !query_params.is_empty() {
            code.push_str("        let mut params = HashMap::new();\n");
            for param in &query_params {
                code.push_str(&format!(
                    "        if let Some(value) = {} {{ params.insert(\"{}\".to_string(), value.to_string()); }}\n",
                    param, param
                ));
            }
        }

        // Build headers
        if !header_params.is_empty() {
            code.push_str("        let mut headers = HashMap::new();\n");
            for param in &header_params {
                code.push_str(&format!(
                    "        if let Some(value) = {} {{ headers.insert(\"{}\".to_string(), value.to_string()); }}\n",
                    param, param
                ));
            }
        }

        // Build the path with parameter substitution if needed
        let path_code = if !path_params.is_empty() {
            // Use simple path formatting without named parameters to avoid redundant argument issue
            let mut path_with_subs = endpoint.path.clone();
            for (i, path_param) in path_params.iter().enumerate() {
                path_with_subs = path_with_subs.replace(&format!("{{{}}}", path_param), &format!("{{{}}}", i));
            }
            format!("&format!(\"{}\", {})", path_with_subs, path_params.join(", "))
        } else {
            format!("\"{}\"", endpoint.path)
        };

        // Make the request
        let method = format!("reqwest::Method::{}", endpoint.method);
        code.push_str(&format!(
            "        self.make_request({}, {}, ",
            method, path_code
        ));

        if !query_params.is_empty() {
            code.push_str("Some(&params), ");
        } else {
            code.push_str("None, ");
        }

        if let Some(body_def) = &endpoint.request_body {
            if body_def.required {
                code.push_str("Some(body), ");
            } else {
                code.push_str("body, ");
            }
        } else {
            code.push_str("None::<&()>, ");
        }

        if !header_params.is_empty() {
            code.push_str("Some(&headers)");
        } else {
            code.push_str("None");
        }

        code.push_str(").await\n    }\n\n");

        Ok(code)
    }

    fn generate_rust_types(&self) -> crate::Result<String> {
        // For now, return empty types - could be enhanced with proper schema generation
        Ok("// TODO: Add Rust types for request/response structures\n".to_string())
    }

    fn rust_type(&self, schema_type: &str) -> &str {
        match schema_type {
            "integer" => "i64",
            "number" => "f64",
            "boolean" => "bool",
            "array" => "Vec<serde_json::Value>",
            "object" => "serde_json::Value",
            _ => "&str",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::parse_openapi_spec;
    use std::path::Path;

    #[test]
    fn test_api_client_creation() {
        // Use one of the example specs for testing
        let spec_path = Path::new("examples/simple-api.json");
        if spec_path.exists() {
            let spec = parse_openapi_spec(spec_path).unwrap();
            let client = ApiClient::new(spec);
            assert!(client.is_ok());
            
            let client = client.unwrap();
            assert!(!client.endpoints.is_empty());
        }
    }

    #[test]
    fn test_typescript_client_generation() {
        // Use one of the example specs for testing
        let spec_path = Path::new("examples/simple-api.json");
        if spec_path.exists() {
            let spec = parse_openapi_spec(spec_path).unwrap();
            let client = ApiClient::new(spec).unwrap();
            let ts_code = client.generate_typescript_client();
            assert!(ts_code.is_ok());
            
            let code = ts_code.unwrap();
            assert!(code.contains("export class ApiClient"));
            assert!(code.contains("makeRequest"));
        }
    }

    #[test]
    fn test_rust_client_generation() {
        // Use one of the example specs for testing
        let spec_path = Path::new("examples/simple-api.json");
        if spec_path.exists() {
            let spec = parse_openapi_spec(spec_path).unwrap();
            let client = ApiClient::new(spec).unwrap();
            let rust_code = client.generate_rust_client();
            assert!(rust_code.is_ok());
            
            let code = rust_code.unwrap();
            assert!(code.contains("pub struct ApiClient"));
            assert!(code.contains("make_request"));
        }
    }
}