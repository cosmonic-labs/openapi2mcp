use openapi2mcp::{Result, backend::native, cli, mcp, openapi};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    log::info!("Starting openapi2mcp conversion");

    let config = cli::parse_args()?;

    log::info!("Converting OpenAPI spec: {:?}", config.input_file);
    log::info!("Output directory: {:?}", config.output_dir);
    log::info!("Target language: {:?}", config.language);

    println!("Converting OpenAPI spec: {:?}", config.input_file);
    println!("Output directory: {:?}", config.output_dir);
    println!("Language: {:?}", config.language);

    let spec = openapi::parse_openapi_spec_from_path(&config.input_file)?;
    log::info!(
        "Parsed OpenAPI spec: {} v{} with {} paths",
        spec.info().title,
        spec.info().version,
        spec.paths().paths.len()
    );

    println!(
        "Parsed OpenAPI spec: {} v{}",
        spec.info().title,
        spec.info().version
    );

    let backend = native::NativeFileBackend;
    
    // For TypeScript, prepare the template first
    let template_dir = if matches!(config.language, cli::Target::TypeScript) {
        let template_path = "./mcp-server-template-ts";
        native::prepare_typescript_template(template_path)?;
        template_path
    } else {
        "unused-for-rust" // Rust doesn't need templates
    };
    
    let generator = mcp::McpGenerator::new(spec, config.language);
    generator.generate(
        &backend,
        template_dir,
        config.output_dir.to_str().unwrap(),
        config.server_name.as_deref()
    )?;

    log::info!("MCP server generated successfully!");
    println!("MCP server generated successfully!");
    Ok(())
}
