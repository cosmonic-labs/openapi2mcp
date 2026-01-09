pub mod codegen_typescript;
pub mod mcp_server;
pub mod template;

mod template_features;

#[cfg(all(target_os = "wasi", target_env = "p2"))]
pub mod wash_plugin;

use std::io;
use std::path::PathBuf;
use std::{fs, path::Path};

use openapiv3::OpenAPI;

pub use crate::codegen_typescript::generate_typescript_code;
use crate::mcp_server::ConverterOptions;
pub use crate::mcp_server::MCPServer;
pub use crate::mcp_server::ToolNameExceededAction;

pub type GenerateOptions = ConverterOptions;

/// Generate MCP server code from an OpenAPI spec
///
/// ## Arguments
/// - `openapi_path`: The path to the OpenAPI specification file.
/// - `project_path`: The path to the project root directory where code will be generated.
pub fn generate(
    openapi_path: impl AsRef<Path>,
    project_path: impl AsRef<Path>,
    options: GenerateOptions,
) -> anyhow::Result<()> {
    let openapi_path = openapi_path.as_ref();
    let project_path = project_path.as_ref();

    log::info!(
        "Generating MCP server from OpenAPI spec spec_path={spec_path}, project_path={project_path}",
        spec_path = openapi_path.to_string_lossy(),
        project_path = project_path.to_string_lossy(),
    );

    let openapi = parse_openapi_spec_from_path(openapi_path)?;
    let mcp_server = MCPServer::from_openapi(openapi, options)?;

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

    let features = template_features::Features {
        auth: mcp_server.oauth2_info.is_some(),
    };
    for path_file in get_all_files_in_dir_recursive(project_path)? {
        // skip if file is not readable as text
        if let Ok(contents) = fs::read_to_string(&path_file) {
            let output = template_features::handle_template_features(&features, &contents);
            fs::write(&path_file, output)?;
        }
    }

    Ok(())
}

fn get_all_files_in_dir_recursive(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    fn visit_dir(dir: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, output)?;
                } else {
                    output.push(path);
                }
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    visit_dir(dir, &mut output)?;
    Ok(output)
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
