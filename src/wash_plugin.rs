mod bindings {
    use super::Plugin;

    wit_bindgen::generate!({ generate_all });

    export!(Plugin);
}

use bindings::exports::wasmcloud::wash::plugin::Guest;
pub use bindings::wasmcloud::wash::types::*;

const FS_ROOT: &str = ".local/share/wash/plugins/fs/openapi2mcp";

pub(crate) struct Plugin;

impl Guest for Plugin {
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
            version: "0.5.0".to_string(),
            command: Some(Command {
                id: "openapi2mcp".into(),
                name: "openapi2mcp".into(),
                description: "Generate MCP server tools from OpenAPI endpoints".into(),
                flags: vec![
                    (
                        "project-path".to_string(),
                        CommandArgument {
                            name: "project-path".to_string(),
                            description: "Path to the project root directory for generation"
                                .to_string(),
                            env: Some("PROJECT_PATH".to_string()),
                            default: Some(".".to_string()),
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
                usage: vec!["wash openapi2mcp <INPUT> --project-path <OUTPUT_DIR>".to_string()],
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

        // Find the "project-path" flag value
        let project_path = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "project-path")
            .and_then(|(_, arg)| arg.value.as_ref())
            .ok_or_else(|| "No project path specified".to_string())?;

        // Get the preopened sandbox directory - this is where we can write files in Wasm
        let preopens = bindings::wasi::filesystem::preopens::get_directories();
        let Some((_descriptor, sandbox_path)) = preopens.get(0) else {
            return Err("No sandbox filesystem available".to_string());
        };

        // The sandbox path is typically mounted at {home_dir}/{FS_ROOT}
        // Copy input file to sandbox via host
        runner.host_exec(
            "cp",
            &[
                input_file.to_string(),
                format!("{home_dir}/{FS_ROOT}/spec.yaml"),
            ],
        )?;

        // Create the directory structure for generation in sandbox via host
        runner.host_exec(
            "mkdir",
            &[
                "-p".to_string(),
                format!("{home_dir}/{FS_ROOT}/generated/src/routes/v1/mcp/tools"),
            ],
        )?;

        // Create placeholder index.ts in sandbox via host
        runner.host_exec(
            "touch",
            &[format!(
                "{home_dir}/{FS_ROOT}/generated/src/routes/v1/mcp/tools/index.ts"
            )],
        )?;

        // Generate into the sandbox (WASM can write here)
        crate::generate(
            format!("{sandbox_path}/spec.yaml"),
            format!("{sandbox_path}/generated"),
        )
        .map_err(|e| format!("failed to generate MCP: {e}"))?;

        // Copy generated src directory from sandbox to target project path via host
        let (_stdout, _stderr) = runner.host_exec(
            "cp",
            &[
                "-Rp".to_string(),
                format!("{home_dir}/{FS_ROOT}/generated/src"),
                project_path.to_string(),
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
                    "http://127.0.0.1:8000/v1/mcp".to_string(),
                ],
            )?;
            Ok("Launched inspector".to_string())
        } else {
            Err("Unknown hook".to_string())
        }
    }
}
