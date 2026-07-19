use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the OpenAPI specification file.
    input: PathBuf,

    /// Path to the project root directory where code will be generated.
    #[arg(long, default_value = ".")]
    project_path: PathBuf,

    /// Regex patterns for tools to include in the MCP server. If empty, all tools will be included.
    #[arg(long)]
    include_tools: Option<String>,

    /// Methods to include in the MCP server. If not provided, all methods will be included.
    #[arg(long, value_delimiter = ',', value_parser = parse_http_method)]
    include_methods: Vec<http::Method>,

    /// Maximum length of the tool name. Default is `DEFAULT_MAX_TOOL_NAME_LENGTH`.
    #[arg(long)]
    max_tool_name_length: Option<u32>,

    /// Skip tool names that exceed the maximum length. Default is `false`.
    /// If true, the tool will be skipped and the next tool will be processed.
    /// If false, the tool throw an error.
    #[arg(long, default_value_t = false)]
    skip_long_tool_names: bool,

    /// Enable OAuth2 authentication.
    #[arg(long, default_value_t = false)]
    oauth2: bool,

    /// The authorization URL to be used for this flow.
    #[arg(long, required_if_eq("oauth2", "true"), requires("oauth2"))]
    oauth2_auth_url: Option<String>,

    /// The token URL to be used for this flow.
    #[arg(long, required_if_eq("oauth2", "true"), requires("oauth2"))]
    oauth2_token_url: Option<String>,

    /// The URL to be used for obtaining refresh tokens.
    #[arg(long, requires("oauth2"))]
    oauth2_refresh_url: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let include_tools = match cli.include_tools {
        Some(r) => Some(Regex::new(&r)?),
        None => None,
    };

    let oauth2_info = cli.oauth2.then(|| openapiv3::AuthorizationCodeOAuth2Flow {
        authorization_url: cli.oauth2_auth_url.unwrap(),
        token_url: cli.oauth2_token_url.unwrap(),
        refresh_url: cli.oauth2_refresh_url,
        scopes: Default::default(),
        extensions: Default::default(),
    });

    openapi2mcp::generate(
        &cli.input,
        &cli.project_path,
        openapi2mcp::GenerateOptions {
            include_tools,
            include_methods: cli.include_methods,
            max_tool_name_length: cli.max_tool_name_length,
            skip_long_tool_names: cli.skip_long_tool_names,
            oauth2_info,
            ..Default::default()
        },
    )
    .expect("failed to generate MCP");
    Ok(())
}

fn parse_http_method(value: &str) -> Result<http::Method, String> {
    http::Method::from_bytes(value.to_ascii_uppercase().as_bytes())
        .map_err(|error| format!("invalid HTTP method `{value}`: {error}"))
}

#[cfg(test)]
mod tests {
    use super::parse_http_method;

    #[test]
    fn parses_http_methods_case_insensitively() {
        assert_eq!(parse_http_method("get").unwrap(), http::Method::GET);
        assert_eq!(parse_http_method("POST").unwrap(), http::Method::POST);
    }

    #[test]
    fn rejects_invalid_http_methods() {
        let error = parse_http_method("not a method").unwrap_err();

        assert!(error.starts_with("invalid HTTP method `not a method`:"));
    }
}
