use openapi2mcp::{Result, cli, mcp, openapi};

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

    let spec = openapi::parse_openapi_spec(&config.input_file)?;
    log::info!(
        "Parsed OpenAPI spec: {} v{} with {} paths",
        spec.info().title, 
        spec.info().version,
        spec.paths().paths.len()
    );
    
    println!(
        "Parsed OpenAPI spec: {} v{}",
        spec.info().title, spec.info().version
    );

    let generator = mcp::McpGenerator::new(spec, config.language);
    generator.generate(&config.output_dir, config.server_name.as_deref())?;

    log::info!("MCP server generated successfully!");
    println!("MCP server generated successfully!");
    Ok(())
}
