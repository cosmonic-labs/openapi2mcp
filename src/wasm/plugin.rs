use crate::{
    openapi,
    wasm::bindings::{
        exports::wasmcloud::wash::plugin::{
            Command, Guest as WashPlugin, HookType, Metadata, Runner,
        },
        wasi::{
            self,
            logging::logging::{Level, log},
        },
        wasmcloud::wash::types::CommandArgument,
    },
};

const FS_ROOT: &str = ".local/share/wash/plugins/fs/openapi2mcp";

pub(crate) struct Plugin;

impl WashPlugin for Plugin {
    /// Called by wash to retrieve the plugin metadata. It's recommended to avoid
    /// any computation or logging in this function as it's called on each command execution.
    fn info() -> Metadata {
        Metadata {
            id: "openapi2mcp".into(),
            name: "openapi2mcp".into(),
            description: "Generate MCP server tools from OpenAPI endpoints".to_string(),
            contact: "Cosmonic Team <team@cosmonic.com>".to_string(),
            url: "https://github.com/cosmonic-labs/openapi2mcp".to_string(),
            license: "Apache-2.0".to_string(),
            version: "0.1.0".to_string(),
            command: Some(Command {
                id: "openapi2mcp".into(),
                name: "openapi2mcp".into(),
                description: "Generate MCP server tools from OpenAPI endpoints".into(),
                flags: vec![
                    (
                        "dir".to_string(),
                        CommandArgument {
                            name: "dir".to_string(),
                            description: "Directory to output the generated MCP files".to_string(),
                            env: Some("OUTPUT_DIR".to_string()),
                            default: None,
                            value: None,
                        },
                    ),
                    (
                        "server-name".to_string(),
                        CommandArgument {
                            name: "server-name".to_string(),
                            description: "Name of the server for the generated MCP".to_string(),
                            env: Some("SERVER_NAME".to_string()),
                            default: Some("my_server".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "language".to_string(),
                        CommandArgument {
                            name: "language".to_string(),
                            description: "Target language for the generated MCP".to_string(),
                            env: Some("LANGUAGE".to_string()),
                            default: Some("typescript".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "home-dir".to_string(),
                        CommandArgument {
                            name: "home-dir".to_string(),
                            description: "Home directory".to_string(),
                            env: Some("HOME".to_string()),
                            default: Some("/home".to_string()),
                            value: None,
                        },
                    ),
                ],
                arguments: vec![CommandArgument {
                    name: "input".to_string(),
                    description: "Path to the OpenAPI specification file".to_string(),
                    env: Some("INPUT_FILE".to_string()),
                    default: None,
                    value: None,
                }],
                usage: vec!["wash openapi2mcp <INPUT> --output <OUTPUT_DIR> [OPTIONS]".to_string()],
            }),
            sub_commands: vec![],
            hooks: vec![HookType::BeforeDev],
        }
    }

    /// Called before any commands or hooks are executed so that the plugin could take preflight actions
    /// such as checking the environment, validating configuration, or preparing resources. Note that
    /// any in-memory state will _not_ be persisted in this component, and the plugin-config store
    /// should be used for any state that needs to be shared across commands or hooks.
    fn initialize(_runner: Runner) -> Result<String, String> {
        Ok(String::with_capacity(0))
    }

    /// Handle the execution of a given command. The resulting value should be the string that will
    /// be printed to the user, or an error message if the command failed.
    fn run(runner: Runner, cmd: Command) -> Result<String, String> {
        // Find the "input" argument value
        let input_file = cmd
            .arguments
            .iter()
            .find(|arg| arg.name == "input")
            .and_then(|arg| arg.value.as_ref())
            .ok_or_else(|| "No input file provided".to_string())?;

        // Find the "home-dir" flag value
        let home_dir = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "home-dir")
            .and_then(|(_, arg)| arg.value.as_ref())
            .ok_or_else(|| "No home directory specified".to_string())?;
        let (openapi_yaml, _stderr) = runner.host_exec("cat", &vec![input_file.to_owned()])?;

        let preopens = wasi::filesystem::preopens::get_directories();
        let Some((descriptor, _path)) = preopens.get(0) else {
            return Err("No sandbox filesystem available".to_string());
        };

        // find output dir in command
        let Some(output_dir) = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "dir")
            .and_then(|(_, arg)| arg.value.as_ref())
        else {
            return Err("No output directory specified".to_string());
        };

        let spec = openapi::parse_openapi_spec(openapi_yaml)
            .map_err(|e| format!("Failed to parse OpenAPI spec: {e}"))?;
        // TODO: cmd flag language get
        let (_stdout, _stderr) = runner.host_exec(
            "git",
            &[
                "clone".to_string(),
                "https://github.com/cosmonic-labs/mcp-server-template-ts".to_string(),
                format!("{home_dir}/{FS_ROOT}/mcp-server-template-ts"),
            ],
        )?;

        let read_dir = descriptor.read_directory().unwrap();
        while let Ok(Some(entry)) = read_dir.read_directory_entry() {
            log(
                Level::Debug,
                "",
                &format!("Found directory entry: {:?}", entry),
            );
        }

        // Use the consolidated wasm module for WASI functionality
        crate::wasm::generator::generate_mcp_project(
            spec,
            crate::cli::Target::TypeScript,
            "mcp-server-template-ts",
            "generated",
            Some("my-server"),
        )
        .map_err(|e| format!("Failed to generate MCP: {e}"))?;

        let (_stdout, _stderr) = runner.host_exec(
            "mv",
            &[
                format!("{home_dir}/{FS_ROOT}/generated"),
                output_dir.to_owned(),
            ],
        )?;

        Ok("MCP server generated successfully".to_string())
    }

    /// Handle the execution of a given hook type. The resulting value should be the string that will
    /// be printed to the user, or an error message if the hook failed.
    fn hook(runner: Runner, hook: HookType) -> Result<String, String> {
        if matches!(hook, HookType::BeforeDev) {
            runner.host_exec_background(
                "npx",
                &[
                    "@modelcontextprotocol/inspector".to_string(),
                    "--transport".to_string(),
                    "http".to_string(),
                    "--server-url".to_string(),
                    "http://127.0.0.1:8000/mcp".to_string(),
                ],
            )?;
            Ok("Launched inspector".to_string())
        } else {
            Err("Unknown hook".to_string())
        }
    }
}
