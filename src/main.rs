use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(long, default_value = ".")]
    project_path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    openapi2mcp::generate(&cli.input, &cli.project_path).expect("failed to generate MCP");
}
