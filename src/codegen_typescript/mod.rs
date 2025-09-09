use crate::mcp_server::{MCPServer, MCPTool, MCPToolPropertyType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct FileCode {
    pub name: String,
    pub code: String,
}

pub fn generate_typescript_code<F>(mcp_server: &MCPServer, file_code: F)
where
    F: Fn(FileCode),
{
    for tool in &mcp_server.tools {
        let code = tool_to_code(mcp_server, tool);
        file_code(FileCode {
            name: tool.name.clone(),
            code: code.unwrap(),
        });
    }
}

fn tool_to_code(server: &MCPServer, tool: &MCPTool) -> anyhow::Result<String> {
    let mut output = String::new();

    // Import statements
    writeln!(output, "import z from \"zod\";")?;
    writeln!(
        output,
        "import {{ McpServer as UpstreamMCPServer }} from \"@modelcontextprotocol/sdk/server/mcp.js\";"
    )?;
    writeln!(
        output,
        "import {{ CallToolResult }} from \"@modelcontextprotocol/sdk/types.js\";"
    )?;

    // Generate Zod schema from tool input schema
    // let zod_schema = self.generate_zod_schema_from_tool(&tool)?;
    let zod_schema = generate_zod_schema_from_tool(&tool)?;

    // Generate setupTool function
    writeln!(
        output,
        "export function setupTool<S extends UpstreamMCPServer>(server: S) {{"
    )?;
    writeln!(output, "  server.tool(")?;
    writeln!(output, "    \"{}\",", tool.name)?;
    writeln!(output, "    \"{}\",", tool.description)?;
    writeln!(output, "    {},", zod_schema)?;
    writeln!(output, "    async (args): Promise<CallToolResult> => {{")?;

    // Generate API call logic
    writeln!(output, "      try {{")?;

    writeln!(
        output,
        "        const response = await fetch('{}{}', {{",
        server.base_url, tool.calls[0].path
    )?;
    writeln!(output, "            headers: {{",)?;
    writeln!(
        output,
        "              'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0',",
    )?;
    writeln!(output, "            }},",)?;
    writeln!(output, "            method: '{}',", tool.calls[0].method)?; // TODO: handle multiple calls
    writeln!(output, "        }}).then(response => response.text());",)?;
    writeln!(output, "")?;

    writeln!(
        output,
        "        const result = {{ success: true, message: \"API call would be made here\" }};",
    )?;
    writeln!(output, "")?;
    writeln!(output, "        return {{")?;
    writeln!(output, "          content: [")?;
    writeln!(output, "            {{")?;
    writeln!(output, "              type: \"text\",")?;
    writeln!(output, "              text: response,")?;

    writeln!(output, "            }},")?;
    writeln!(output, "          ],")?;
    writeln!(output, "        }};")?;
    writeln!(output, "      }} catch (error) {{")?;
    writeln!(
        output,
        "        console.error(`Error executing {}:`, error);",
        tool.name
    )?;
    writeln!(output, "        return {{")?;
    writeln!(output, "          content: [")?;
    writeln!(output, "            {{")?;
    writeln!(output, "              type: \"text\",")?;
    writeln!(
        output,
        "              text: `Error executing {}: ${{error instanceof Error ? error.message : String(error)}}`,",
        tool.name
    )?;
    writeln!(output, "            }},")?;
    writeln!(output, "          ],")?;
    writeln!(output, "        }};")?;
    writeln!(output, "      }}")?;
    writeln!(output, "    }},")?;
    writeln!(output, "  );")?;
    writeln!(output, "}}")?;

    Ok(output)
}

fn generate_zod_schema_from_tool(tool: &MCPTool) -> anyhow::Result<String> {
    let mut zod_fields = String::new();
    for property in &tool.properties {
        let mut zod_type = match property.type_ {
            MCPToolPropertyType::String => "z.string()",
            MCPToolPropertyType::Number => "z.number()",
            MCPToolPropertyType::Boolean => "z.boolean()",
        }
        .to_string();

        // Add description if present
        if let Some(description) = &property.description {
            zod_type = format!(
                "{}.describe(\"{}\")",
                zod_type,
                description.replace('"', "\\\"")
            );
        }

        if !property.required {
            zod_type = format!("{}.optional()", zod_type);
        }

        writeln!(zod_fields, "      {}: {},", property.name, zod_type)?;
    }

    Ok(format!("{{\n{}    }}", zod_fields))
}
