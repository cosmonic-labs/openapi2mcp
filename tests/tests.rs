#[cfg(test)]
mod tests {
    use openapi2mcp::GenerateOptions;
    use regex::Regex;
    use std::fs;
    use std::path::Path;

    fn test_generate(name: &str, options: GenerateOptions) {
        // Check for both .yaml and .json input files
        let yaml_path = format!("./tests/{name}/input.yaml");
        let json_path = format!("./tests/{name}/input.json");
        let openapi_path = if Path::new(&yaml_path).exists() {
            yaml_path
        } else if Path::new(&json_path).exists() {
            json_path
        } else {
            panic!("No input.yaml or input.json found for test {}", name);
        };

        let project_path = format!("./tests/{name}/generated/");

        // Create the expected project structure for testing
        let tools_dir = format!("{project_path}src/routes/v1/mcp/tools/");
        fs::create_dir_all(&tools_dir).unwrap();

        // Create the index.ts file that the generator expects
        fs::write(
            format!("{tools_dir}index.ts"),
            "// Placeholder for testing\n",
        )
        .unwrap();

        // Create the constants.ts file that the generator expects
        fs::write(
            format!("{project_path}src/constants.ts"),
            "// Placeholder for testing\n",
        )
        .unwrap();

        openapi2mcp::generate(&openapi_path, &project_path, options).unwrap();
    }

    #[test]
    fn weather_gov() {
        test_generate("weather-gov", Default::default());
    }

    #[test]
    fn adobe_firefly() {
        test_generate("adobe-firefly", Default::default());
    }

    #[test]
    fn microsoft_graph() {
        test_generate(
            "microsoft-graph",
            GenerateOptions {
                include_tools: Some(
                    Regex::new("drives/\\{drive-id\\}|me/drive|me/mail|me/calendar|me/chats")
                        .unwrap(),
                ),
                include_methods: vec![http::Method::GET],
                skip_long_tool_names: true,
                oauth2_info: Some(openapiv3::AuthorizationCodeOAuth2Flow {
                    authorization_url:
                        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                    token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token"
                        .to_string(),
                    refresh_url: None,
                    scopes: Default::default(),
                    extensions: Default::default(),
                }),
                ..Default::default()
            },
        );
    }
}
