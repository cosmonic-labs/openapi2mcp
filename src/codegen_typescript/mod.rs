use crate::mcp_server::{MCPServer, MCPTool, MCPToolPropertyType};
use std::{collections::HashSet, fmt::Write};

#[derive(Debug, Clone)]
pub struct FileCode {
    pub name: String,
    pub code: String,
}

pub fn generate_typescript_code<F>(mcp_server: &MCPServer, file_code: F) -> anyhow::Result<()>
where
    F: Fn(FileCode) -> anyhow::Result<()>,
{
    for tool in &mcp_server.tools {
        let code = tool_to_code(tool)?;
        file_code(FileCode {
            name: tool.name.clone(),
            code,
        })?;
    }

    Ok(())
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
    writeln!(output, "  const params = {zod_schema};")?;
    writeln!(output, "  type ParamsType = {{")?;
    writeln!(
        output,
        "    [K in keyof typeof params]: z.infer<typeof params[K]>"
    )?;
    writeln!(output, "  }};")?;
    writeln!(output, "  server.tool(")?;
    writeln!(output, "    \"{}\",", comment(&tool.name))?;
    writeln!(output, "    \"{}\",", comment(&tool.description))?;
    writeln!(output, "    params,")?;
    writeln!(
        output,
        "    async (args: ParamsType): Promise<CallToolResult> => {{"
    )?;

    // Generate API call logic
    writeln!(output, "      try {{")?;
    writeln!(output, "        const response = await httpClient.call({{")?;

    writeln!(output, "          path: `{}`,", tool.call.path)?;
    writeln!(output, "          method: '{}',", tool.call.method)?;

    if !tool.call.path_params.is_empty() {
        writeln!(output, "          pathParams: {{")?;
        for (key, value) in &tool.call.path_params {
            writeln!(output, "            \"{key}\": args.{value},")?;
        }
        writeln!(output, "          }},")?;
    }

    if !tool.call.query.is_empty() {
        writeln!(output, "          query: {{")?;
        for (key, value) in &tool.call.query {
            writeln!(output, "            \"{key}\": args.{value},")?;
        }
        writeln!(output, "          }},")?;
    }

    if !tool.call.headers.is_empty() {
        writeln!(output, "          headers: {{")?;
        for (key, value) in &tool.call.headers {
            writeln!(output, "            \"{key}\": args.{value},")?;
        }
        writeln!(output, "          }},")?;
    }
    writeln!(output, "        }})")?;

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
        let mut prefix = String::from("    ");
        if visited.contains(&property.name) {
            writeln!(
                prefix,
                "// TODO: the following property name already exists"
            )?;
            write!(prefix, "    // ")?;
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

    Ok(format!("{{\n{}  }}", zod_fields))
}

fn comment(s: &str) -> String {
    s.replace("\n", "\\n")
}
