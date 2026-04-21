mod cli;
mod detection;
mod downloads;
mod metadata;
mod setup;
mod utils;
mod validation;

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

    fn validate_and_use_fqbn<F>(&self, fqbn: &str, action: F)
    where
        F: FnOnce(&str),
    {
        if let Err(e) = cli::validate_fqbn(fqbn) {
            eprintln!("Arduino: {}", e);
        } else {
            action(fqbn);
        }
    }

    fn extract_library_paths(worktree: &zed::Worktree) -> Option<Vec<String>> {
        let lsp_settings = LspSettings::for_worktree("arduino", worktree).ok()?;
        let settings = lsp_settings.settings?;
        let library_paths = settings.get("libraryPaths")?;
        let paths_array = library_paths.as_array()?;

        let paths: Vec<String> = paths_array
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect();

        if paths.is_empty() {
            None
        } else {
            Some(paths)
        }
    }

    fn ensure_clangd_available(&mut self, args: &mut Vec<String>, worktree: &zed::Worktree) {
        if let Some(clangd_path) = detection::find_clangd(worktree) {
            self.installation_state
                .record_clangd_from_system(clangd_path.clone(), metadata::ToolSource::ZedManaged);
            self.installation_state.save().ok();
            args.push("-clangd".to_string());
            args.push(clangd_path);
        } else {
            match downloads::get_clangd_binary(worktree, &mut self.cached_clangd_path) {
                Ok(clangd_path) => {
                    let version = utils::extract_clangd_version(&clangd_path)
                        .unwrap_or_else(|| "unknown".to_string());
                    self.installation_state
                        .record_clangd_download(&version, clangd_path.clone());
                    self.installation_state.save().ok();
                    args.push("-clangd".to_string());
                    args.push(clangd_path);
                }
                Err(e) => {
                    eprintln!("\n{}", e);
                    eprintln!("\nArduino Extension will continue without clangd. IntelliSense features will be limited.");
                    eprintln!("Basic syntax highlighting and compilation will still work.\n");
                }
            }
        }
    }

    fn ensure_arduino_cli_available(
        &mut self,
        args: &mut Vec<String>,
        worktree: &zed::Worktree,
    ) -> Result<()> {
        if let Some(cli_path) = worktree.which("arduino-cli") {
            self.installation_state
                .record_arduino_cli_from_path(cli_path.clone());
            self.installation_state.save().ok();
            args.push("-cli".to_string());
            args.push(cli_path);
        } else {
            match downloads::get_arduino_cli_binary(worktree, &mut self.cached_arduino_cli_path) {
                Ok(cli_path) => {
                    let version = utils::extract_arduino_cli_version(&cli_path)
                        .unwrap_or_else(|| "unknown".to_string());
                    self.installation_state
                        .record_arduino_cli_download(&version, cli_path.clone());
                    self.installation_state.save().ok();

                    setup::create_isolated_arduino_config(&self.installation_state).ok();

                    args.push("-cli".to_string());
                    args.push(cli_path);
                }
                Err(e) => {
                    eprintln!("\n{}", e);
                    eprintln!("\nArduino Extension cannot start without arduino-cli.");
                    return Err("arduino-cli is required but could not be obtained. See error message above for recovery options.".to_string());
                }
            }
        }
        Ok(())
    }

    fn setup_automation(&mut self, args: &[String], worktree: &zed::Worktree) {
        // Extract FQBN once and reuse it
        let fqbn = utils::get_arg_value(args, "-fqbn").map(|s| s.to_string());

        // Auto-install core if enabled and FQBN is specified
        if utils::get_setting(worktree, "autoInstallCore", false) {
            if let Some(ref fqbn) = fqbn {
                self.validate_and_use_fqbn(fqbn, |fqbn| {
                    if let Some(core_id) = cli::extract_core_id(fqbn) {
                        if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                            if !cli::is_core_installed(cli_path, &core_id) {
                                let config_path = utils::get_arg_value(args, "-cli-config");
                                if cli::install_core(cli_path, &core_id, config_path).is_ok() {
                                    eprintln!("Arduino: Installed core {} automatically", core_id);
                                }
                            }
                        }
                    }
                });
            }
        }

        // Auto-generate compilation database if enabled
        if utils::get_setting(worktree, "autoGenerateCompileDb", false)
            && !detection::check_compilation_database(worktree)
        {
            if let Some(ref fqbn) = fqbn {
                self.validate_and_use_fqbn(fqbn, |fqbn| {
                    if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                        let config_path = utils::get_arg_value(args, "-cli-config");
                        let library_paths = utils::get_library_paths(worktree);
                        if cli::generate_compilation_database(
                            cli_path,
                            fqbn,
                            config_path,
                            &library_paths,
                            worktree,
                        )
                        .is_ok()
                        {
                            eprintln!("Arduino: Generated compile_commands.json automatically");
                        }
                    }
                });
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

        // Check dependencies and report any issues
        validation::report_dependencies(worktree);

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

        if !utils::has_arg(&args, "-clangd") {
            self.ensure_clangd_available(&mut args, worktree);
        }

        if !utils::has_arg(&args, "-cli") {
            self.ensure_arduino_cli_available(&mut args, worktree)?;
        }

        // Auto-detect or auto-create arduino-cli config if not specified
        let user_specified_cli_config = utils::has_arg(&args, "-cli-config");
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

        let user_specified_libraries = utils::has_arg(&args, "-libraries");
        if !user_specified_libraries {
            if let Some(paths) = Self::extract_library_paths(worktree) {
                args.push("-libraries".to_string());
                args.push(paths.join(","));
                eprintln!("Arduino: Using custom library paths: {}", paths.join(", "));
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
