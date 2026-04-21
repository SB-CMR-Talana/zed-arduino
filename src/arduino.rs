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

        // Auto-detect or auto-create arduino-cli config if not specified
        let user_specified_cli_config = args.iter().any(|arg| arg == "-cli-config");
        if !user_specified_cli_config {
            if let Some(config_path) = detection::find_arduino_cli_config(worktree) {
                args.push("-cli-config".to_string());
                args.push(config_path);
            } else if utils::get_setting(worktree, "autoCreateConfig", false) {
                // Auto-create minimal config if enabled
                let config_path = format!("{}/.arduino-cli.yaml", worktree.root_path());
                if std::fs::write(&config_path, "board_manager:\n  additional_urls: []\n").is_ok() {
                    args.push("-cli-config".to_string());
                    args.push(config_path);
                }
            }
        }

        // Auto-install core if enabled and FQBN is specified
        let user_specified_fqbn = args.iter().any(|arg| arg == "-fqbn");
        if user_specified_fqbn && utils::get_setting(worktree, "autoInstallCore", false) {
            if let Some(fqbn_idx) = args.iter().position(|a| a == "-fqbn") {
                if let Some(fqbn) = args.get(fqbn_idx + 1) {
                    if let Some(core_id) = cli::extract_core_id(fqbn) {
                        let cli_path = args
                            .iter()
                            .position(|a| a == "-cli")
                            .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()));

                        if let Some(cli) = cli_path {
                            if !cli::is_core_installed(cli, &core_id) {
                                let config_path = args
                                    .iter()
                                    .position(|a| a == "-cli-config")
                                    .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()));

                                // Try to install core (ignore errors as they're not critical)
                                cli::install_core(cli, &core_id, config_path).ok();
                            }
                        }
                    }
                }
            }
        }

        // Auto-generate compilation database if enabled
        if user_specified_fqbn
            && utils::get_setting(worktree, "autoGenerateCompileDb", false)
            && !detection::check_compilation_database(worktree)
        {
            let fqbn = args
                .iter()
                .position(|a| a == "-fqbn")
                .and_then(|idx| args.get(idx + 1).cloned());

            let cli_path = args
                .iter()
                .position(|a| a == "-cli")
                .and_then(|idx| args.get(idx + 1).cloned());

            let config_path = args
                .iter()
                .position(|a| a == "-cli-config")
                .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()));

            if let (Some(fqbn), Some(cli)) = (fqbn, cli_path) {
                // Try to generate (ignore errors as they're not critical)
                cli::generate_compilation_database(&cli, &fqbn, config_path, worktree).ok();
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
