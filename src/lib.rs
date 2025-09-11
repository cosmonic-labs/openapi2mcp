pub mod codegen_typescript;
pub mod mcp_server;
mod shell;
pub mod template;

use std::{fs, path::Path};

use openapiv3::OpenAPI;

pub use crate::codegen_typescript::generate_typescript_code;
pub use crate::mcp_server::MCPServer;

pub fn generate(openapi_path: impl AsRef<Path>, generated_path: impl AsRef<Path>) {
    let openapi_path = openapi_path.as_ref();
    let generated_path = generated_path.as_ref();
    let openapi = parse_openapi_spec_from_path(openapi_path).unwrap();

    let mcp_server = MCPServer::from_openapi(openapi).unwrap();

    let _ = fs::remove_dir_all(&generated_path);
    template::clone_template(&generated_path);
    let tools_path = format!("{}/src/routes/v1/mcp/tools/", generated_path.display());
    generate_typescript_code(&mcp_server, |file_code| {
        let file_path = format!(
            "{tools_path}{}.ts",
            file_code.name.replace('/', " ").trim().replace(' ', "_")
        );

        fs::create_dir_all(&tools_path).unwrap();
        fs::write(file_path, file_code.code).unwrap();
    });

    template::update_tools_index_ts(&mcp_server, &generated_path).unwrap();
    template::update_constants_ts(&mcp_server, &generated_path).unwrap();
}

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
