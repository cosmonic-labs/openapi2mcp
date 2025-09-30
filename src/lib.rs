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
/// - `project_path`: The path to the project root directory where code will be generated.
pub fn generate(
    openapi_path: impl AsRef<Path>,
    project_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let openapi_path = openapi_path.as_ref();
    let project_path = project_path.as_ref();

    log::info!(
        "Generating MCP server from OpenAPI spec spec_path={spec_path}, project_path={project_path}",
        spec_path = openapi_path.to_string_lossy(),
        project_path = project_path.to_string_lossy(),
    );

    // Validate that the project structure exists
    let tools_index_path = project_path.join("src/routes/v1/mcp/tools/index.ts");
    if !tools_index_path.exists() {
        return Err(anyhow::anyhow!(
            "Project structure validation failed: {} does not exist.\n\
             Please ensure you have the base MCP server project structure set up before running code generation.",
            tools_index_path.display()
        ));
    }

    let openapi = parse_openapi_spec_from_path(openapi_path)?;
    let mcp_server = MCPServer::from_openapi(openapi)?;

    let tools_code_path = project_path.join("src/routes/v1/mcp/tools/");
    generate_typescript_code(&mcp_server, |file_code| {
        let file_path = tools_code_path.join(format!(
            "{}.ts",
            file_code.name.replace('/', " ").trim().replace(' ', "_")
        ));

        fs::write(file_path, file_code.code)?;
        Ok(())
    })?;

    // Remove placeholder file `/tools/echo.ts` if it exists
    let echo_path = tools_code_path.join("echo.ts");
    if echo_path.exists() {
        fs::remove_file(echo_path)?;
    }

    template::update_tools_index_ts(&mcp_server, &project_path)?;
    template::update_constants_ts(&mcp_server, &project_path)?;

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
