pub mod codegen_typescript;
pub mod mcp_server;

use std::{fs, path::Path};

use openapiv3::OpenAPI;

pub use crate::mcp_server::MCPServer;

pub fn parse_openapi_spec_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<OpenAPI> {
    let content = fs::read_to_string(&path)?;

    let extension = path.as_ref().extension().and_then(|s| s.to_str());
    let openapi: OpenAPI = match extension {
        Some("json") => serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))?,
        _ => Err(anyhow::anyhow!("Unsupported file extension"))?,
    };

    validate_spec(&openapi)?;
    Ok(openapi)
}

fn validate_spec(spec: &OpenAPI) -> anyhow::Result<()> {
    if !spec.openapi.starts_with("3.") {
        Err(anyhow::anyhow!(
            "Only OpenAPI 3.x specifications are supported"
        ))?;
    }

    if spec.info.title.is_empty() {
        Err(anyhow::anyhow!("API title is required"))?;
    }

    if spec.paths.paths.is_empty() {
        Err(anyhow::anyhow!("OpenAPI spec must have at least one path"))?;
    }

    // Validate that all references can be resolved
    for (_, path_item_ref) in &spec.paths.paths {
        if let openapiv3::ReferenceOr::Reference { reference } = path_item_ref {
            Err(anyhow::anyhow!(
                "Path item references are not supported: {}",
                reference
            ))?;
        }
    }

    Ok(())
}
