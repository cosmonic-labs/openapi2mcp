use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Integration tests for the openapi2mcp CLI tool
/// These tests run the actual CLI binary and verify end-to-end functionality

// TODO fix this test
#[test]
#[ignore = "WIP"]
fn test_simple_api_typescript_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("simple-ts");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
            "-n",
            "t",
            "tests/fixtures/mcp-server-template-ts",
            "simple-tasks",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify generated files exist
    assert!(output_path.join("package.json").exists());
    assert!(output_path.join("tsconfig.json").exists());

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

// TODO fix this test
#[test]
#[ignore = "WIP"]
fn test_ecommerce_api_comprehensive() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("ecommerce");

    println!("Output path: {:?}", output_path);

    let output = Command::new("cargo")
        .args([
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

    println!(
        "Command stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // there is a ts file for each tool at src/routes/v1/mcp/tools
    let tools_dir = output_path
        .join("src")
        .join("routes")
        .join("v1")
        .join("mcp")
        .join("tools");
    assert!(
        tools_dir.exists(),
        "Tools directory does not exist: {:?}",
        tools_dir
    );
    let tool_files: Vec<_> = fs::read_dir(&tools_dir)
        .expect("Failed to read tools directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert!(!tool_files.is_empty(), "No tool files found");
    let tool_file_names: Vec<_> = tool_files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    println!("Found tool files: {:?}", tool_file_names);
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("listProducts")),
        "listProducts tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("createOrder")),
        "createOrder tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("getOrderById")),
        "getOrderById tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("updateOrderStatus")),
        "updateOrderStatus tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("listCustomers")),
        "listCustomers tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("getCustomerById")),
        "getCustomerById tool not found"
    );
    assert!(
        tool_file_names
            .iter()
            .any(|name| name.contains("createCustomer")),
        "createCustomer tool not found"
    );
}

#[test]
fn test_slack_api_form_encoded() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("slack");

    let output = Command::new("cargo")
        .args([
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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
        .args([
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
        .args([
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
        .args(["run", "--", "--help"])
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
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0")); // Version from Cargo.toml
}

#[test]
fn test_default_output_directory() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-l",
            "typescript",
            "-t",
            "tests/fixtures/mcp-server-template-ts",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
        .args([
            "run",
            "--",
            "-i",
            "examples/simple-api.json",
            "-o",
            output_path.to_str().unwrap(),
            "-l",
            "typescript",
            "-t",
            "tests/fixtures/mcp-server-template-ts",
            "-n",
            "custom-server-name",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
            .args([
                "run",
                "--",
                "-i",
                example,
                "-o",
                output_path.to_str().unwrap(),
                "-l",
                "typescript",
                "-t",
                "tests/fixtures/mcp-server-template-ts",
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
