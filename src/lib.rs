pub mod codegen_typescript;
pub mod mcp_server;
mod shell;
pub mod template;

use std::{fs, path::Path};

use openapiv3::OpenAPI;

pub use crate::codegen_typescript::generate_typescript_code;
pub use crate::mcp_server::MCPServer;

/// Generate MCP server code from an OpenAPI spec
///
/// ## Arguments
/// - `openapi_path`: The path to the OpenAPI specification file.
/// - `generated_path`: The path to the directory where template will be cloned and placed.
/// - `tools_path`: The path to the directory where tool code will be placed. Usually the same as `generated_path`
/// - `template_repository`: Optional override template repository URL.
/// - `runner`: The command runner to use when executing in Wasm
pub fn generate(
    openapi_path: impl AsRef<Path>,
    generated_path: impl AsRef<Path>,
    tools_path: impl AsRef<Path>,
    template_repository: Option<String>,
    runner: Option<&crate::shell::Runner>,
) -> anyhow::Result<()> {
    log::info!(
        "Generating MCP server from OpenAPI spec spec_path={spec_path}, template_path={template_path}",
        spec_path = openapi_path.as_ref().to_string_lossy(),
        template_path = generated_path.as_ref().to_string_lossy(),
    );

    let openapi_path = openapi_path.as_ref();
    let generated_path = generated_path.as_ref();
    let openapi = parse_openapi_spec_from_path(openapi_path)?;

    let mcp_server = MCPServer::from_openapi(openapi)?;

    let _ = fs::remove_dir_all(&generated_path);
    template::clone_template(template_repository, &generated_path, runner)?;
    let tools_code_path = format!("{}/src/routes/v1/mcp/tools/", tools_path.as_ref().display());
    generate_typescript_code(&mcp_server, |file_code| {
        let file_path = format!(
            "{tools_code_path}{}.ts",
            file_code.name.replace('/', " ").trim().replace(' ', "_")
        );

        fs::create_dir_all(&tools_path)?;
        fs::write(file_path, file_code.code)?;
        Ok(())
    })?;

    // Remove placeholder file `/tools/echo.ts`
    fs::remove_file(format!("{tools_code_path}echo.ts"))?;
    template::update_tools_index_ts(&mcp_server, &tools_path)?;
    template::update_constants_ts(&mcp_server, &tools_path)?;

    Ok(())
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
