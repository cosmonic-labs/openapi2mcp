use clap::{Arg, Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub input_file: PathBuf,
    pub output_dir: PathBuf,
    pub server_name: Option<String>,
    pub language: Target,
    pub template_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Target {
    TypeScript,
    Rust,
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "typescript" | "ts" => Ok(Target::TypeScript),
            "rust" => Ok(Target::Rust),
            _ => Err(format!("Unknown target: {}", s)),
        }
    }
}

pub fn build_cli() -> Command {
    Command::new("openapi2mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Convert OpenAPI specifications to MCP servers")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Path to OpenAPI specification file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for generated MCP server")
                .default_value("./output"),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name for the generated MCP server"),
        )
        .arg(
            Arg::new("language")
                .short('l')
                .long("language")
                .value_name("LANGUAGE")
                .help("Target language for MCP server")
                .default_value("typescript")
                .value_parser(clap::value_parser!(Target)),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .value_name("DIR")
                .help("Path to TypeScript template directory (for TypeScript generation only)"),
        )
}

pub fn parse_args() -> crate::Result<Config> {
    let matches = build_cli().get_matches();

    let input_file = matches
        .get_one::<String>("input")
        .unwrap()
        .parse::<PathBuf>()
        .map_err(|_| crate::Error::Parse("Invalid input file path".to_string()))?;

    let output_dir = matches
        .get_one::<String>("output")
        .unwrap()
        .parse::<PathBuf>()
        .map_err(|_| crate::Error::Parse("Invalid output directory path".to_string()))?;

    let server_name = matches.get_one::<String>("name").cloned();
    let language = matches.get_one::<Target>("language").unwrap().clone();
    let template_dir = matches
        .get_one::<String>("template")
        .map(|s| s.parse::<PathBuf>())
        .transpose()
        .map_err(|_| crate::Error::Parse("Invalid template directory path".to_string()))?;

    Ok(Config {
        input_file,
        output_dir,
        server_name,
        language,
        template_dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_str() {
        assert!(matches!(
            "typescript".parse::<Target>().unwrap(),
            Target::TypeScript
        ));
        assert!(matches!(
            "ts".parse::<Target>().unwrap(),
            Target::TypeScript
        ));
        assert!(matches!("rust".parse::<Target>().unwrap(), Target::Rust));

        assert!("invalid".parse::<Target>().is_err());
        assert!("python".parse::<Target>().is_err());
    }

    #[test]
    fn test_target_debug() {
        let ts_target = Target::TypeScript;
        let rust_target = Target::Rust;

        assert_eq!(format!("{:?}", ts_target), "TypeScript");
        assert_eq!(format!("{:?}", rust_target), "Rust");
    }

    #[test]
    fn test_config_debug() {
        let config = Config {
            input_file: "/path/to/spec.yaml".into(),
            output_dir: "/path/to/output".into(),
            server_name: Some("test-server".to_string()),
            language: Target::TypeScript,
            template_dir: None,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("spec.yaml"));
        assert!(debug_str.contains("test-server"));
        assert!(debug_str.contains("TypeScript"));
    }

    #[test]
    fn test_build_cli_has_required_args() {
        let app = build_cli();
        let args = app.get_arguments().collect::<Vec<_>>();

        let input_arg = args.iter().find(|arg| arg.get_id() == "input").unwrap();
        assert!(input_arg.is_required_set());

        let output_arg = args.iter().find(|arg| arg.get_id() == "output").unwrap();
        assert!(!output_arg.is_required_set());

        let language_arg = args.iter().find(|arg| arg.get_id() == "language").unwrap();
        assert!(!language_arg.is_required_set());
    }

    #[test]
    fn test_cli_help_contains_expected_text() {
        let mut app = build_cli();
        let help = app.render_help();
        let help_str = help.to_string();

        assert!(help_str.contains("Convert OpenAPI specifications to MCP servers"));
        assert!(help_str.contains("--input"));
        assert!(help_str.contains("--output"));
        assert!(help_str.contains("--language"));
        assert!(help_str.contains("--template"));
        assert!(help_str.contains("typescript"));
    }
}
