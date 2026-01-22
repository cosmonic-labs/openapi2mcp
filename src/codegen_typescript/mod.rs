use crate::mcp_server::{
    MCPServer, MCPTool, MCPToolProperty, MCPToolPropertyRequired, MCPToolPropertyType, Value,
    ValueSource,
};
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
        let code = tool_to_code(mcp_server, tool)?;
        file_code(FileCode {
            name: tool.name.clone(),
            code,
        })?;
    }

    Ok(())
}

fn tool_to_code(mcp_server: &MCPServer, tool: &MCPTool) -> anyhow::Result<String> {
    let mut output = String::new();

    // Import statements
    writeln!(output, "import z from \"zod\";")?;
    writeln!(
        output,
        "import {{ McpServer }} from \"@modelcontextprotocol/sdk/server/mcp.js\";"
    )?;
    writeln!(
        output,
        "import {{ CallToolResult, ServerRequest, ServerNotification }} from \"@modelcontextprotocol/sdk/types.js\";"
    )?;
    writeln!(
        output,
        "import {{ RequestHandlerExtra }} from \"@modelcontextprotocol/sdk/shared/protocol.js\";"
    )?;
    writeln!(
        output,
        "import {{ httpClient }} from \"../../../../http_client\";"
    )?;

    // Generate Zod schema from tool input schema
    let zod_schema = generate_zod_schema_from_tool(&tool)?;

    // Generate setupTool function
    writeln!(output, "export function setupTool(server: McpServer) {{")?;
    writeln!(output, "  const params = {zod_schema};")?;
    writeln!(
        output,
        "  type ParamsType = z.infer<z.ZodObject<typeof params>>;"
    )?;
    writeln!(output, "  server.tool(")?;
    writeln!(output, "    \"{}\",", comment(&tool.name))?;
    writeln!(output, "    \"{}\",", comment(&tool.description))?;
    writeln!(output, "    params,")?;
    writeln!(
        output,
        "    async (args: ParamsType, context: RequestHandlerExtra<ServerRequest, ServerNotification>): Promise<CallToolResult> => {{"
    )?;

    // Generate API call logic
    writeln!(output, "      try {{")?;
    writeln!(output, "        const response = await httpClient.call({{")?;

    writeln!(output, "          path: `{}`,", tool.call.path)?;
    writeln!(output, "          method: '{}',", tool.call.method)?;

    if let Some(_oauth2_info) = &mcp_server.oauth2_info {
        writeln!(
            output,
            "          authorizationHeader: context.requestInfo?.headers[\"authorization\"]?.toString(),"
        )?;
    }

    fn display_value(value: &ValueSource) -> String {
        match value {
            ValueSource::Fixed(value) => match value {
                Value::Boolean(_) | Value::Number(_) => format!("{value}.toString()"),
                Value::String(value) => format!("\"{value}\""),
            },
            ValueSource::Property(property) => format!("args.{property}?.toString()"),
        }
    }

    if !tool.call.path_params.is_empty() {
        writeln!(output, "          pathParams: {{")?;
        for (key, value) in &tool.call.path_params {
            writeln!(output, "            \"{key}\": {},", display_value(value))?;
        }
        writeln!(output, "          }},")?;
    }

    if !tool.call.query.is_empty() {
        writeln!(output, "          query: {{")?;
        for (key, value) in &tool.call.query {
            writeln!(output, "            \"{key}\": {},", display_value(value))?;
        }
        writeln!(output, "          }},")?;
    }

    if !tool.call.headers.is_empty() {
        writeln!(output, "          headers: {{")?;
        for (key, value) in &tool.call.headers {
            writeln!(output, "            \"{key}\": {},", display_value(value))?;
        }
        writeln!(output, "          }},")?;
    }

    if let Some(body) = &tool.call.body {
        writeln!(output, "          body: {},", display_value(body))?;
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
        visited.insert(property.name.clone());

        let zod_type = mcp_tool_property_to_zod_type(property, 2)?;

        writeln!(zod_fields, "    {}: {}", property.name, zod_type)?;
    }

    Ok(format!("{{\n{}  }}", zod_fields))
}

fn mcp_tool_property_to_zod_type(
    property: &MCPToolProperty,
    indentation: usize,
) -> anyhow::Result<String> {
    let ind_str = " ".repeat(indentation * 2);
    let mut output = String::new();
    match &property.type_ {
        MCPToolPropertyType::String => write!(output, "z.string()")?,
        MCPToolPropertyType::Number => write!(output, "z.number()")?,
        MCPToolPropertyType::Boolean => write!(output, "z.boolean()")?,
        MCPToolPropertyType::Array(property) => {
            writeln!(output, "z.array(")?;
            write!(
                output,
                "{ind_str}  {}",
                mcp_tool_property_to_zod_type(property, indentation + 1)?
            )?;
            write!(output, "{ind_str})")?;
        }
        MCPToolPropertyType::Object(hash_map) => {
            writeln!(output, "z.object({{")?;
            for (name, type_) in hash_map.iter() {
                write!(
                    output,
                    "{ind_str}  \"{}\": {}",
                    name,
                    mcp_tool_property_to_zod_type(type_, indentation + 1)?
                )?;
            }
            write!(output, "{ind_str}}})")?;
        }
    };

    match &property.required {
        MCPToolPropertyRequired::Default(_) | MCPToolPropertyRequired::Optional => {
            write!(output, ".optional()")?;
        }
        MCPToolPropertyRequired::Required => {}
    }

    if let Some(description) = &property.description {
        write!(output, ".describe(\"{}\")", comment(description))?;
    }

    write!(output, ",")?;

    Ok(output)
}

fn comment(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace("\n", "\\n")
        .replace("\"", "\\\"")
}
