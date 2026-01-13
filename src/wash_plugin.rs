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
                    (
                        "include-tools".to_string(),
                        CommandArgument {
                            name: "include-tools".to_string(),
                            description: "Regex patterns for tools to include in the MCP server. If empty, all tools will be included.".to_string(),
                            env: Some("OPENAPI2MCP_INCLUDE_TOOLS".to_string()),
                            default: Some("".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "include-methods".to_string(),
                        CommandArgument {
                            name: "include-methods".to_string(),
                            description: "Methods to include in the MCP server. If not provided, all methods will be included.".to_string(),
                            env: Some("OPENAPI2MCP_INCLUDE_METHODS".to_string()),
                            default: Some("".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "max-tool-name-length".to_string(),
                        CommandArgument {
                            name: "max-tool-name-length".to_string(),
                            description: "Maximum length of the tool name. Default is `DEFAULT_MAX_TOOL_NAME_LENGTH`.".to_string(),
                            env: Some("OPENAPI2MCP_MAX_TOOL_NAME_LENGTH".to_string()),
                            default: Some("".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "skip-long-tool-names".to_string(),
                        CommandArgument {
                            name: "skip-long-tool-names".to_string(),
                            description: "Skip tool names that exceed the maximum length. Default is `false`. If true, the tool will be skipped and the next tool will be processed. If false, the tool will throw an error.".to_string(),
                            env: Some("OPENAPI2MCP_skip_long_tool_names".to_string()),
                            default: Some("fail".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "oauth2".to_string(),
                        CommandArgument {
                            name: "oauth2".to_string(),
                            description: "Enable OAuth2 authentication.".to_string(),
                            env: Some("OPENAPI2MCP_OAUTH2".to_string()),
                            default: Some("false".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "oauth2-auth-url".to_string(),
                        CommandArgument {
                            name: "oauth2-auth-url".to_string(),
                            description: "The authorization URL to be used for this flow.".to_string(),
                            env: Some("OPENAPI2MCP_OAUTH2_AUTH_URL".to_string()),
                            default: Some("".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "oauth2-token-url".to_string(),
                        CommandArgument {
                            name: "oauth2-token-url".to_string(),
                            description: "The token URL to be used for this flow.".to_string(),
                            env: Some("OPENAPI2MCP_OAUTH2_TOKEN_URL".to_string()),
                            default: Some("".to_string()),
                            value: None,
                        },
                    ),
                    (
                        "oauth2-refresh-url".to_string(),
                        CommandArgument {
                            name: "oauth2-refresh-url".to_string(),
                            description: "The URL to be used for obtaining refresh tokens.".to_string(),
                            env: Some("OPENAPI2MCP_OAUTH2_REFRESH_URL".to_string()),
                            default: Some("".to_string()),
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

        // Find the "include-methods" flag value
        let include_methods = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "include-methods")
            .and_then(|(_, arg)| arg.value.as_ref())
            .ok_or_else(|| "No include methods specified".to_string())?;
        let include_methods = match include_methods.is_empty() {
            true => Vec::new(),
            false => {
                let mut methods = Vec::new();
                for method in include_methods.split(',') {
                    let method =
                        http::Method::from_bytes(method.as_bytes()).map_err(|e| e.to_string())?;
                    methods.push(method);
                }
                methods
            }
        };

        // Find the "include-tools" flag value
        let include_tools = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "include-tools")
            .and_then(|(_, arg)| arg.value.as_ref())
            .ok_or_else(|| "No include tools specified".to_string())?;
        let include_tools = if include_tools.is_empty() {
            None
        } else {
            Some(regex::Regex::new(include_tools).map_err(|e| e.to_string())?)
        };

        // Find the "skip-long-tool-names" flag value
        let skip_long_tool_names = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "skip-long-tool-names")
            .and_then(|(_, arg)| arg.value.as_ref().map(|s| s.to_lowercase()))
            .ok_or_else(|| "No tool name exceeded action specified".to_string())?;
        let skip_long_tool_names = skip_long_tool_names.to_lowercase() == "true";

        // Find the "oauth2" flag value
        let oauth2 = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "oauth2")
            .and_then(|(_, arg)| arg.value.as_ref())
            .ok_or_else(|| "No oauth2 flag specified".to_string())?
            .to_lowercase();
        let oauth2 = oauth2 == "true" || oauth2 == "1";

        // Find the "oauth2-auth-url" flag value
        let oauth2_auth_url = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "oauth2-auth-url")
            .and_then(|(_, arg)| arg.value.clone())
            .ok_or_else(|| "No oauth2-auth-url flag specified".to_string())?;

        // Find the "oauth2-token-url" flag value
        let oauth2_token_url = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "oauth2-token-url")
            .and_then(|(_, arg)| arg.value.clone())
            .ok_or_else(|| "No oauth2-token-url flag specified".to_string())?;

        // Find the "oauth2-refresh-url" flag value
        let oauth2_refresh_url = cmd
            .flags
            .iter()
            .find(|(name, _)| name == "oauth2-refresh-url")
            .and_then(|(_, arg)| arg.value.clone())
            .ok_or_else(|| "No oauth2-refresh-url flag specified".to_string())?;

        if !oauth2
            && (!oauth2_auth_url.is_empty()
                || !oauth2_token_url.is_empty()
                || !oauth2_refresh_url.is_empty())
        {
            return Err(
                "OAuth2 authentication is not enabled, but OAuth2 configuration is provided"
                    .to_string(),
            );
        }

        if oauth2 && (oauth2_auth_url.is_empty() || oauth2_token_url.is_empty()) {
            return Err(
                "OAuth2 authentication is enabled, but OAuth2 configuration is not provided"
                    .to_string(),
            );
        }

        // Get the preopened sandbox directory - this is where we can write files in Wasm
        // TODO remove, when we have wash volumeMounts, this is just getting the home dir path for the plugin
        let preopens = bindings::wasi::filesystem::preopens::get_directories();
        let Some((_descriptor, sandbox_path)) = preopens.get(0) else {
            return Err("No sandbox filesystem available".to_string());
        };

        // The sandbox path is typically mounted at {home_dir}/{FS_ROOT}
        let sandbox_generated = format!("{home_dir}/{FS_ROOT}/generated");

        // Cleanup any previous sandbox state
        let _ = runner.host_exec("rm", &["-rf".to_string(), sandbox_generated.clone()]);

        // Copy the entire project into the sandbox so we can read and modify template files
        runner.host_exec(
            "cp",
            &[
                "-Rp".to_string(),
                project_path.clone(),
                sandbox_generated.clone(),
            ],
        )?;

        // Copy input file to sandbox via host
        runner.host_exec(
            "cp",
            &[
                input_file.to_string(),
                format!("{home_dir}/{FS_ROOT}/spec.yaml"),
            ],
        )?;

        // Ensure the tools directory exists in the sandbox
        runner.host_exec(
            "mkdir",
            &[
                "-p".to_string(),
                format!("{sandbox_generated}/src/routes/v1/mcp/tools"),
            ],
        )?;

        let oauth2_info = if oauth2 {
            Some(openapiv3::AuthorizationCodeOAuth2Flow {
                authorization_url: oauth2_auth_url,
                token_url: oauth2_token_url,
                refresh_url: if oauth2_refresh_url.is_empty() {
                    None
                } else {
                    Some(oauth2_refresh_url)
                },
                scopes: Default::default(),
                extensions: Default::default(),
            })
        } else {
            None
        };

        // Generate into the sandbox (WASM can read and modify template files here)
        crate::generate(
            format!("{sandbox_path}/spec.yaml"),
            format!("{sandbox_path}/generated"),
            crate::GenerateOptions {
                include_tools,
                include_methods,
                skip_long_tool_names,
                oauth2_info,
                ..Default::default()
            },
        )
        .map_err(|e| format!("failed to generate MCP: {e}"))?;

        // Copy modified project back from sandbox to target project path
        // Using rsync-like copy to update files in place
        runner.host_exec(
            "cp",
            &[
                "-Rp".to_string(),
                format!("{sandbox_generated}/."),
                format!("{project_path}/"),
            ],
        )?;

        // Cleanup the sandbox directory via host
        runner.host_exec("rm", &["-rf".to_string(), sandbox_generated])?;

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
