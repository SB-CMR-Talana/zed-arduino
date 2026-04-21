mod cli;
mod detection;
mod downloads;
mod utils;

use std::collections::HashMap;
use zed_extension_api::{self as zed, serde_json, settings::LspSettings, LanguageServerId, Result};

struct ArduinoExtension {
    cached_language_server_path: Option<String>,
    cached_arduino_cli_path: Option<String>,
}

impl ArduinoExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // Check for explicit path override in settings
        if let Ok(lsp_settings) = LspSettings::for_worktree("arduino", worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(path) = binary.path {
                    return Ok(path.clone());
                }
            }
        }

        // Use downloads module to get the binary (checks PATH, cache, then downloads)
        downloads::get_language_server_binary(
            language_server_id,
            worktree,
            &mut self.cached_language_server_path,
        )
    }
}

impl zed::Extension for ArduinoExtension {
    fn new() -> Self {
        Self {
            cached_language_server_path: None,
            cached_arduino_cli_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Auto-generate .zed/settings.json if it doesn't exist and feature is enabled
        utils::auto_generate_project_settings(worktree).ok();

        // Get args and env from LSP settings first
        let mut args: Vec<String> = Vec::new();
        let mut env: HashMap<String, String> = HashMap::new();

        if let Ok(lsp_settings) = LspSettings::for_worktree("arduino", worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(binary_args) = binary.arguments {
                    args = binary_args;
                }

                if let Some(binary_env) = binary.env {
                    env = binary_env;
                }
            }
        }

        // Get the path to the language server binary
        let command_path = self.language_server_binary_path(language_server_id, worktree)?;

        // Check if the user already specified the -clangd flag in settings
        let user_specified_clangd = args.iter().any(|arg| arg == "-clangd");
        let user_specified_cli = args.iter().any(|arg| arg == "-cli");

        if !user_specified_clangd {
            // Use detection module to find clangd automatically
            if let Some(clangd_path) = detection::find_clangd(worktree) {
                args.push("-clangd".to_string());
                args.push(clangd_path);
            }
        }

        if !user_specified_cli {
            // Try to find arduino-cli in PATH first
            if let Some(cli_path) = worktree.which("arduino-cli") {
                args.push("-cli".to_string());
                args.push(cli_path);
            } else {
                // If not in PATH, try downloading it
                if let Ok(cli_path) =
                    downloads::get_arduino_cli_binary(worktree, &mut self.cached_arduino_cli_path)
                {
                    args.push("-cli".to_string());
                    args.push(cli_path);
                }
            }
        }

        // Determine environment variables.
        // Always start with shell_env so PATH, HOME, etc. are present,
        // then let any user-specified env vars override those defaults.
        let default_env = match zed::current_platform().0 {
            zed::Os::Mac | zed::Os::Linux => worktree.shell_env(),
            zed::Os::Windows => Vec::new(),
        };
        // Insert shell_env first, then user settings override
        let mut merged_env: HashMap<String, String> = default_env.into_iter().collect();
        merged_env.extend(env);
        env = merged_env;

        Ok(zed::Command {
            command: command_path,
            args,
            env: env.into_iter().collect(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // This function provides the `workspace/configuration` response to the language server
        // Get the 'settings' section from the arduino LSP settings in Zed
        let settings = LspSettings::for_worktree("arduino", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();

        Ok(Some(settings))
    }
}

zed::register_extension!(ArduinoExtension);
