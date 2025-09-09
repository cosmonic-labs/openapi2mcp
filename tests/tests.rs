#[cfg(test)]
mod tests {
    fn test_generate(name: &str) {
        let openapi_path = format!("./tests/{name}/input.yaml");
        let generated_path = format!("./tests/{name}/generated/");
        openapi2mcp::generate(&openapi_path, &generated_path);
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
