mod cli;
mod detection;
mod downloads;
mod metadata;
mod setup;
mod utils;

use std::collections::HashMap;
use zed_extension_api::{self as zed, serde_json, settings::LspSettings, LanguageServerId, Result};

struct ArduinoExtension {
    cached_language_server_path: Option<String>,
    cached_arduino_cli_path: Option<String>,
    cached_clangd_path: Option<String>,
    installation_state: metadata::InstallationState,
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
                    // Manual path specified by user
                    self.installation_state
                        .record_language_server_manual(path.clone());
                    self.installation_state.save().ok();
                    return Ok(path.clone());
                }
            }
        }

        // Use downloads module to get the binary (checks PATH, cache, then downloads)
        let path = downloads::get_language_server_binary(
            language_server_id,
            worktree,
            &mut self.cached_language_server_path,
        )?;

        // Track that we downloaded it
        if let Some(version) = utils::extract_language_server_version(&path) {
            self.installation_state
                .record_language_server_download(&version, path.clone());
        } else {
            self.installation_state
                .record_language_server_download("unknown", path.clone());
        }
        self.installation_state.save().ok();

        Ok(path)
    }

    /// Setup automation features (auto-install core, auto-generate compile DB)
    fn setup_automation(&mut self, args: &[String], worktree: &zed::Worktree) {
        // Extract FQBN once and reuse it
        let fqbn = utils::get_arg_value(args, "-fqbn").map(|s| s.to_string());

        // Auto-install core if enabled and FQBN is specified
        if utils::get_setting(worktree, "autoInstallCore", false) {
            if let Some(ref fqbn) = fqbn {
                // Validate FQBN format before using it
                if let Err(e) = cli::validate_fqbn(fqbn) {
                    eprintln!("Arduino: {}", e);
                } else if let Some(core_id) = cli::extract_core_id(fqbn) {
                    if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                        if !cli::is_core_installed(cli_path, &core_id) {
                            let config_path = utils::get_arg_value(args, "-cli-config");
                            // Try to install core (ignore errors - not critical)
                            if cli::install_core(cli_path, &core_id, config_path).is_ok() {
                                eprintln!("Arduino: Installed core {} automatically", core_id);
                            }
                        }
                    }
                }
            }
        }

        // Auto-generate compilation database if enabled
        if utils::get_setting(worktree, "autoGenerateCompileDb", false)
            && !detection::check_compilation_database(worktree)
        {
            if let Some(ref fqbn) = fqbn {
                // Validate FQBN format before using it
                if let Err(e) = cli::validate_fqbn(fqbn) {
                    eprintln!("Arduino: {}", e);
                } else if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                    let config_path = utils::get_arg_value(args, "-cli-config");
                    // Try to generate (ignore errors - not critical)
                    if cli::generate_compilation_database(cli_path, fqbn, config_path, worktree)
                        .is_ok()
                    {
                        eprintln!("Arduino: Generated compile_commands.json automatically");
                    }
                }
            }
        }
    }
}

impl zed::Extension for ArduinoExtension {
    fn new() -> Self {
        Self {
            cached_language_server_path: None,
            cached_arduino_cli_path: None,
            cached_clangd_path: None,
            installation_state: metadata::InstallationState::load(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Detect and record platform on first run
        if self.installation_state.get_platform().is_none() {
            let (platform, _) = zed::current_platform();
            let detected_platform = match platform {
                zed::Os::Linux => metadata::Platform::Linux,
                zed::Os::Mac => metadata::Platform::MacOS,
                zed::Os::Windows => metadata::Platform::Windows,
            };
            self.installation_state.record_platform(detected_platform);
            self.installation_state.save().ok();
        }

        // Auto-generate .zed/settings.json if it doesn't exist and feature is enabled
        setup::auto_generate_project_settings(worktree).ok();

        // Auto-generate .zed/tasks.json if it doesn't exist and feature is enabled
        setup::auto_generate_tasks(worktree, &self.installation_state).ok();

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
            // Try to find clangd in PATH or Zed-managed locations first
            if let Some(clangd_path) = detection::find_clangd(worktree) {
                // Found in system - track it
                self.installation_state.record_clangd_from_system(
                    clangd_path.clone(),
                    metadata::ToolSource::ZedManaged,
                );
                self.installation_state.save().ok();
                args.push("-clangd".to_string());
                args.push(clangd_path);
            } else {
                // If not found, try downloading it
                match downloads::get_clangd_binary(worktree, &mut self.cached_clangd_path) {
                    Ok(clangd_path) => {
                        // We downloaded it - track it
                        if let Some(version) = utils::extract_clangd_version(&clangd_path) {
                            self.installation_state
                                .record_clangd_download(&version, clangd_path.clone());
                        } else {
                            self.installation_state
                                .record_clangd_download("unknown", clangd_path.clone());
                        }
                        self.installation_state.save().ok();
                        args.push("-clangd".to_string());
                        args.push(clangd_path);
                    }
                    Err(e) => {
                        eprintln!(
                            "Arduino: Failed to get clangd: {}. IntelliSense may be limited.",
                            e
                        );
                    }
                }
            }
        }

        if !user_specified_cli {
            // Try to find arduino-cli in PATH first
            if let Some(cli_path) = worktree.which("arduino-cli") {
                // Found in PATH - use system installation
                self.installation_state
                    .record_arduino_cli_from_path(cli_path.clone());
                self.installation_state.save().ok();
                args.push("-cli".to_string());
                args.push(cli_path);
            } else {
                // If not in PATH, try downloading it
                if let Ok(cli_path) =
                    downloads::get_arduino_cli_binary(worktree, &mut self.cached_arduino_cli_path)
                {
                    // We downloaded it - use isolated data directory
                    if let Some(version) = crate::utils::extract_arduino_cli_version(&cli_path) {
                        self.installation_state
                            .record_arduino_cli_download(&version, cli_path.clone());
                    } else {
                        self.installation_state
                            .record_arduino_cli_download("unknown", cli_path.clone());
                    }
                    self.installation_state.save().ok();

                    // Create isolated config pointing to extension directory
                    setup::create_isolated_arduino_config(&self.installation_state).ok();

                    args.push("-cli".to_string());
                    args.push(cli_path);
                }
            }
        }

        // Auto-detect or auto-create arduino-cli config if not specified
        let user_specified_cli_config = args.iter().any(|arg| arg == "-cli-config");
        if !user_specified_cli_config {
            // If we downloaded arduino-cli, use isolated config
            if self.installation_state.arduino_cli_uses_isolated_data() {
                let isolated_config = "arduino-cli-isolated.yaml";
                args.push("-cli-config".to_string());
                args.push(isolated_config.to_string());
            } else if let Some(config_path) = detection::find_arduino_cli_config(worktree) {
                // Use system config
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

        // Run automation features (auto-install cores, auto-generate compile DB)
        self.setup_automation(&args, worktree);

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
