use std::fmt::Write;
use std::path::Path;

use crate::MCPServer;

// TODO: handle this as string instead of file
pub fn update_tools_index_ts(
    server: &MCPServer,
    template_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let template_dir = template_dir.as_ref();
    let tools_index_path = format!(
        "{}/src/routes/v1/mcp/tools/index.ts",
        template_dir.display()
    );

    let mut code = String::new();
    // TODO: use std::fmt::Writeln
    code.push_str("import { MCPServer } from \"../server\";\n\n");

    // Import all generated tools
    for tool in &server.tools {
        code.push_str(&format!(
            "import * as {} from \"./{}\";\n",
            tool.name, tool.name
        ));
    }

    code.push_str("\nexport function setupAllTools(server: MCPServer) {\n");

    // Call setupTool for each tool
    for tool in &server.tools {
        code.push_str(&format!("  {}.setupTool(server);\n", tool.name));
    }

    code.push_str("}\n");

    std::fs::write(tools_index_path, &code)?;

    log::info!("Updated tools index with {} tools", server.tools.len());
    Ok(())
}

// TODO: handle this as string instead of file
pub fn update_constants_ts(
    server: &MCPServer,
    template_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let template_dir = template_dir.as_ref();
    let tools_index_path = format!("{}/src/constants.ts", template_dir.display());

    let mut code = String::new();

    writeln!(code, "export const API_BASE_URL = \"{}\";", server.base_url)?;

    if let Some(oauth2_info) = &server.oauth2_info {
        writeln!(
            code,
            "export const OAUTH_AUTHORIZATION_URL = \"{}\";",
            oauth2_info.authorization_url
        )?;
        writeln!(
            code,
            "export const OAUTH_TOKEN_URL = \"{}\";",
            oauth2_info.token_url
        )?;
    }

    std::fs::write(tools_index_path, &code)?;
    Ok(())
}
