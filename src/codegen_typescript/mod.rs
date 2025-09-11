use crate::mcp_server::{MCPServer, MCPTool, MCPToolPropertyType};
use std::{collections::HashSet, fmt::Write};

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
        let code = tool_to_code(tool);
        file_code(FileCode {
            name: tool.name.clone(),
            code: code.unwrap(),
        });
    }
}

fn tool_to_code(tool: &MCPTool) -> anyhow::Result<String> {
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
    writeln!(
        output,
        "import {{ httpClient }} from \"../../../../http_client.js\";"
    )?;

    // Generate Zod schema from tool input schema
    let zod_schema = generate_zod_schema_from_tool(&tool)?;

    // Generate setupTool function
    writeln!(
        output,
        "export function setupTool<S extends UpstreamMCPServer>(server: S) {{"
    )?;
    writeln!(output, "  server.tool(")?;
    writeln!(output, "    \"{}\",", comment(&tool.name))?;
    writeln!(output, "    \"{}\",", comment(&tool.description))?;
    writeln!(output, "    {},", zod_schema)?;
    // TODO: don't use any, declare real type
    writeln!(
        output,
        "    async (args: any): Promise<CallToolResult> => {{"
    )?;

    // Generate API call logic
    writeln!(output, "      try {{")?;
    writeln!(output, "        const response = await httpClient.call({{")?;
    writeln!(output, "          path: `/alerts/active/zone/{{zoneId}}`,")?;
    writeln!(output, "          pathParams: {{")?;
    for (key, value) in &tool.calls[0].path_params {
        writeln!(output, "            \"{key}\": args.{value},")?;
    }
    writeln!(output, "          }},")?;
    writeln!(output, "          method: 'GET',")?;
    writeln!(output, "          headers: {{")?;
    // TODO: remove this header
    writeln!(
        output,
        "            \"User-Agent\": \"Mozilla/5.0 (X11; Linux x86_64; rv:142.0) Gecko/20100101 Firefox/142.0\","
    )?;
    writeln!(output, "          }}")?;
    writeln!(output, "        }})")?;
    // TODO: don't use any, declare real type
    writeln!(
        output,
        "        .then((response: Response) => response.text());"
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

    let mut visited = HashSet::new();
    for property in &tool.properties {
        let mut prefix = String::from("      ");
        if visited.contains(&property.name) {
            writeln!(
                prefix,
                "// TODO: the following property name already exists"
            )?;
            write!(prefix, "      // ")?;
        }
        visited.insert(property.name.clone());

        let mut zod_type = match property.type_ {
            MCPToolPropertyType::String => "z.string()",
            MCPToolPropertyType::Number => "z.number()",
            MCPToolPropertyType::Boolean => "z.boolean()",
        }
        .to_string();

        // Add description if present
        if let Some(description) = &property.description {
            zod_type = format!("{}.describe(\"{}\")", zod_type, comment(description));
        }

        if !property.required {
            zod_type = format!("{}.optional()", zod_type);
        }

        writeln!(zod_fields, "{}\"{}\": {},", prefix, property.name, zod_type)?;
    }

    Ok(format!("{{\n{}    }}", zod_fields))
}

fn comment(s: &str) -> String {
    s.replace("\n", "\\n")
}
