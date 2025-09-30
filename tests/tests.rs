#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn test_generate(name: &str) {
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
            "// Placeholder for testing\n"
        ).unwrap();

        // Create the constants.ts file that the generator expects
        fs::write(
            format!("{project_path}src/constants.ts"),
            "// Placeholder for testing\n"
        ).unwrap();

        openapi2mcp::generate(&openapi_path, &project_path).unwrap();
    }

    #[test]
    fn weather_api() {
        test_generate("weather-api");
    }

    #[test]
    fn weather_gov() {
        test_generate("weather-gov");
    }

    #[test]
    fn adobe_firefly() {
        test_generate("adobe-firefly");
    }
}
