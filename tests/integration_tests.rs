use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Integration tests for the openapi2mcp CLI tool
/// These tests run the actual CLI binary and verify end-to-end functionality

#[test]
fn test_simple_api_typescript_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("simple-ts");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
            "-n",
            "simple-tasks",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify generated files exist
    assert!(output_path.join("package.json").exists());
    assert!(output_path.join("tsconfig.json").exists());
    assert!(output_path.join("src").join("index.ts").exists());

    // Verify package.json content
    let package_json = fs::read_to_string(output_path.join("package.json")).unwrap();
    assert!(package_json.contains("simple-tasks"));
    assert!(package_json.contains("@modelcontextprotocol/sdk"));
    assert!(package_json.contains("1.2.0")); // Version from simple-api.json

    // Verify TypeScript file contains expected tools
    let index_ts = fs::read_to_string(output_path.join("src").join("index.ts")).unwrap();
    assert!(index_ts.contains("getAllTasks"));
    assert!(index_ts.contains("createTask"));
    assert!(index_ts.contains("getTaskById"));
    assert!(index_ts.contains("updateTaskStatus"));
    assert!(index_ts.contains("ListToolsRequestSchema"));
    assert!(index_ts.contains("CallToolRequestSchema"));
}

#[test]
fn test_petstore_rust_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("petstore-rust");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/petstore.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "rust",
            "-n",
            "petstore-api",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify generated files exist
    assert!(output_path.join("Cargo.toml").exists());
    assert!(output_path.join("src").join("main.rs").exists());

    // Verify Cargo.toml content
    let cargo_toml = fs::read_to_string(output_path.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("petstore-api"));
    assert!(cargo_toml.contains("1.0.0")); // Version from petstore.yaml
    assert!(cargo_toml.contains("serde"));
    assert!(cargo_toml.contains("tokio"));
    assert!(cargo_toml.contains("anyhow"));

    // Verify Rust file contains expected tools
    let main_rs = fs::read_to_string(output_path.join("src").join("main.rs")).unwrap();
    assert!(main_rs.contains("listPets"));
    assert!(main_rs.contains("createPet"));
    assert!(main_rs.contains("getPetById"));
    assert!(main_rs.contains("updatePet"));
    assert!(main_rs.contains("deletePet"));
    assert!(main_rs.contains("call_tool"));
    assert!(main_rs.contains("list_tools"));

    // Verify generated Rust code compiles
    let compile_output = Command::new("cargo")
        .args(&["check"])
        .current_dir(&output_path)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        compile_output.status.success(),
        "Generated Rust code failed to compile: {}",
        String::from_utf8_lossy(&compile_output.stderr)
    );
}

#[test]
fn test_weather_api_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("weather");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/weather-api.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify files exist
    assert!(output_path.join("package.json").exists());
    assert!(output_path.join("src").join("index.ts").exists());

    // Verify tool generation from weather API
    let index_ts = fs::read_to_string(output_path.join("src").join("index.ts")).unwrap();
    assert!(index_ts.contains("getCurrentWeather"));
    assert!(index_ts.contains("getWeatherForecast"));
    assert!(index_ts.contains("getHistoricalWeather"));

    // Check for query parameters in generated schema
    assert!(index_ts.contains("location"));
    assert!(index_ts.contains("units"));
    assert!(index_ts.contains("days"));
}

#[test]
fn test_github_api_complex_schemas() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("github");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/github-api.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "rust",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    let main_rs = fs::read_to_string(output_path.join("src").join("main.rs")).unwrap();
    
    // Verify complex operations are handled
    assert!(main_rs.contains("getAuthenticatedUser"));
    assert!(main_rs.contains("listUserRepos"));
    assert!(main_rs.contains("createUserRepo"));
    assert!(main_rs.contains("getRepository"));
    assert!(main_rs.contains("updateRepository"));
    assert!(main_rs.contains("listRepoIssues"));
    assert!(main_rs.contains("createIssue"));

    // Verify path parameters are handled
    assert!(main_rs.contains("owner"));
    assert!(main_rs.contains("repo"));

    // Verify query parameters with enums
    assert!(main_rs.contains("type"));
    assert!(main_rs.contains("sort"));
    assert!(main_rs.contains("direction"));
}

#[test]
fn test_ecommerce_api_comprehensive() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("ecommerce");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/ecommerce-api.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    let index_ts = fs::read_to_string(output_path.join("src").join("index.ts")).unwrap();
    
    // Verify e-commerce operations
    assert!(index_ts.contains("listProducts"));
    assert!(index_ts.contains("createProduct"));
    assert!(index_ts.contains("getProduct"));
    assert!(index_ts.contains("updateProduct"));
    assert!(index_ts.contains("deleteProduct"));
    assert!(index_ts.contains("getCustomerOrders"));
    assert!(index_ts.contains("createOrder"));
    assert!(index_ts.contains("updateOrderStatus"));

    // Verify complex filtering parameters
    assert!(index_ts.contains("category"));
    assert!(index_ts.contains("min_price"));
    assert!(index_ts.contains("max_price"));
    assert!(index_ts.contains("in_stock"));

    // Verify UUID path parameters
    assert!(index_ts.contains("productId"));
    assert!(index_ts.contains("customerId"));
    assert!(index_ts.contains("orderId"));
}

#[test]
fn test_slack_api_form_encoded() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("slack");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/slack-api.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "rust",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    let main_rs = fs::read_to_string(output_path.join("src").join("main.rs")).unwrap();
    
    // Verify Slack API operations
    assert!(main_rs.contains("listConversations"));
    assert!(main_rs.contains("createConversation"));
    assert!(main_rs.contains("postMessage"));
    assert!(main_rs.contains("getConversationHistory"));
    assert!(main_rs.contains("listUsers"));
    assert!(main_rs.contains("getUserInfo"));

    // Verify specific Slack API parameters
    assert!(main_rs.contains("channel"));
    assert!(main_rs.contains("text"));
    assert!(main_rs.contains("cursor"));
    assert!(main_rs.contains("limit"));
}

#[test]
fn test_invalid_openapi_spec() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("invalid");

    // Create an invalid OpenAPI spec
    let invalid_spec_path = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_spec_path, "invalid: yaml: content:").unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            invalid_spec_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with invalid spec
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Parse error") || stderr.contains("Failed to parse"));
}

#[test]
fn test_missing_input_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("missing");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "nonexistent-file.yaml",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with file not found
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IO error") || stderr.contains("No such file"));
}

#[test]
fn test_cli_help_output() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Convert OpenAPI specifications to MCP servers"));
    assert!(stdout.contains("--input"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--language"));
    assert!(stdout.contains("Target language for MCP server"));
    assert!(stdout.contains("typescript")); // default value is shown
}

#[test]
fn test_version_output() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0")); // Version from Cargo.toml
}

#[test]
fn test_default_output_directory() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    
    // Should create default output directory
    assert!(Path::new("./output").exists());
    
    // Clean up
    if Path::new("./output").exists() {
        std::fs::remove_dir_all("./output").ok();
    }
}

#[test]
fn test_server_name_override() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("named");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
            "-n",
            "custom-server-name",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let package_json = fs::read_to_string(output_path.join("package.json")).unwrap();
    assert!(package_json.contains("custom-server-name"));
}

#[test]
fn test_all_example_files_parse() {
    // Test that all example files can be parsed without errors
    let examples = [
        "examples/simple-api.json",
        "examples/petstore.yaml",
        "examples/weather-api.yaml",
        "examples/github-api.yaml",
        "examples/ecommerce-api.yaml",
        "examples/slack-api.yaml",
    ];

    for example in &examples {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = temp_dir.path().join("test");

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--",
                "-i",
                example,
                "-o",
                output_path.to_str().unwrap(),
                "-l",
                "typescript",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(
            output.status.success(),
            "Failed to parse {}: {}",
            example,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_both_targets_generate_different_outputs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ts_output = temp_dir.path().join("ts");
    let rust_output = temp_dir.path().join("rust");

    // Generate TypeScript version
    let ts_result = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            ts_output.to_str().unwrap(),
            "-l",
            "typescript",
        ])
        .output()
        .expect("Failed to execute TypeScript generation");

    assert!(ts_result.status.success());

    // Generate Rust version
    let rust_result = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            rust_output.to_str().unwrap(),
            "-l",
            "rust",
        ])
        .output()
        .expect("Failed to execute Rust generation");

    assert!(rust_result.status.success());

    // Verify different file structures
    assert!(ts_output.join("package.json").exists());
    assert!(ts_output.join("tsconfig.json").exists());
    assert!(ts_output.join("src").join("index.ts").exists());

    assert!(rust_output.join("Cargo.toml").exists());
    assert!(rust_output.join("src").join("main.rs").exists());

    // Verify TypeScript-specific content
    let ts_content = fs::read_to_string(ts_output.join("src").join("index.ts")).unwrap();
    assert!(ts_content.contains("@modelcontextprotocol/sdk"));
    assert!(ts_content.contains("async () =>"));

    // Verify Rust-specific content
    let rust_content = fs::read_to_string(rust_output.join("src").join("main.rs")).unwrap();
    assert!(rust_content.contains("HashMap"));
    assert!(rust_content.contains("async fn"));
}