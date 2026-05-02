// Arduino extension for Zed: manages Language Server Protocol integration,
// tool detection/installation, and automated project setup for Arduino development.

mod arduino_cli;
mod clangd;
mod language_server;
mod metadata;
mod setup;
mod sketches;
mod slash_commands;
mod tools;
mod utils;

use std::collections::HashMap;
use zed_extension_api::{self as zed, serde_json, settings::LspSettings, LanguageServerId, Result};

struct ArduinoExtension {
    cached_language_server_path: Option<String>,
    cached_arduino_cli_path: Option<String>,
    cached_clangd_path: Option<String>,
    // Cached detection results
    cached_clangd_info: Option<tools::ToolInfo>,
    cached_arduino_cli_info: Option<tools::ToolInfo>,
    installation_state: metadata::InstallationState,
}

impl ArduinoExtension {
    // ============================================================================
    // Core Tool Setup
    // ============================================================================

    // Get language server binary path from settings, PATH, cache, or download
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // Check for explicit path override in binary.path (native Zed, takes precedence)
        if let Ok(lsp_settings) = LspSettings::for_worktree("arduino", worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(path) = binary.path {
                    self.installation_state
                        .record_language_server_manual(path.clone());
                    if let Err(e) = self.installation_state.save() {
                        eprintln!("Arduino: Failed to save installation state: {}", e);
                    }
                    return Ok(path.clone());
                }
            }
        }

        // Check for ls.path fallback in settings
        let ls_path = utils::get_string_setting(worktree, "ls.path", "");
        if !ls_path.is_empty() {
            self.installation_state
                .record_language_server_manual(ls_path.clone());
            if let Err(e) = self.installation_state.save() {
                eprintln!("Arduino: Failed to save installation state: {}", e);
            }
            return Ok(ls_path);
        }

        // Use language_server module to get binary (checks PATH, cache, then downloads)
        let path = language_server::get_or_download(
            language_server_id,
            worktree,
            &mut self.cached_language_server_path,
        )?;

        // Track downloaded version
        if let Some(version) = language_server::extract_version(&path) {
            self.installation_state
                .record_language_server_download(&version, path.clone());
        } else {
            self.installation_state
                .record_language_server_download("unknown", path.clone());
        }
        if let Err(e) = self.installation_state.save() {
            eprintln!("Arduino: Failed to save installation state: {}", e);
        }

        Ok(path)
    }

    // Add clangd arguments from settings to args vector
    fn add_clangd_arguments(&self, args: &mut Vec<String>, worktree: &zed::Worktree) {
        let clangd_args = utils::get_string_array_setting(worktree, "clangd.arguments");
        for arg in clangd_args {
            args.push(arg);
        }
    }

    // Find or download clangd and add to args
    fn ensure_clangd_available(&mut self, args: &mut Vec<String>, worktree: &zed::Worktree) {
        // Check cache first - reuse if still valid
        if let Some(ref info) = self.cached_clangd_info {
            if tools::file_exists(&info.path) {
                args.push("-clangd".to_string());
                args.push(info.path.clone());
                self.add_clangd_arguments(args, worktree);
                return;
            } else {
                // Cached tool no longer exists, clear cache
                self.cached_clangd_info = None;
            }
        }

        if let Some(info) = clangd::find(worktree) {
            tools::log_tool_info("clangd", &info);
            self.installation_state
                .record_clangd_from_system(info.path.clone(), metadata::ToolSource::ZedManaged);
            if let Err(e) = self.installation_state.save() {
                eprintln!("Arduino: Failed to save installation state: {}", e);
            }
            args.push("-clangd".to_string());
            args.push(info.path.clone());
            self.cached_clangd_info = Some(info);
            self.add_clangd_arguments(args, worktree);
        } else {
            match clangd::get_or_download(worktree, &mut self.cached_clangd_path) {
                Ok(clangd_path) => {
                    let version = clangd::extract_version(&clangd_path)
                        .unwrap_or_else(|| "unknown".to_string());
                    self.installation_state
                        .record_clangd_download(&version, clangd_path.clone());
                    if let Err(e) = self.installation_state.save() {
                        eprintln!("Arduino: Failed to save installation state: {}", e);
                    }
                    args.push("-clangd".to_string());
                    args.push(clangd_path);
                    self.add_clangd_arguments(args, worktree);
                }
                Err(e) => {
                    eprintln!("\n{}", e);
                    eprintln!("\nArduino Extension will continue without clangd. IntelliSense features will be limited.");
                    eprintln!("Basic syntax highlighting and compilation will still work.\n");
                }
            }
        }
    }

    // Find or download arduino-cli and add to args
    fn ensure_arduino_cli_available(
        &mut self,
        args: &mut Vec<String>,
        worktree: &zed::Worktree,
    ) -> Result<()> {
        // Check cache first - reuse if still valid
        if let Some(ref info) = self.cached_arduino_cli_info {
            if tools::file_exists(&info.path) {
                args.push("-cli".to_string());
                args.push(info.path.clone());
                return Ok(());
            } else {
                // Cached tool no longer exists, clear cache
                self.cached_arduino_cli_info = None;
            }
        }

        if let Some(info) = arduino_cli::find(worktree) {
            tools::log_tool_info("arduino-cli", &info);
            self.installation_state
                .record_arduino_cli_from_path(info.path.clone());
            if let Err(e) = self.installation_state.save() {
                eprintln!("Arduino: Failed to save installation state: {}", e);
            }
            args.push("-cli".to_string());
            args.push(info.path.clone());
            self.cached_arduino_cli_info = Some(info);
        } else {
            match arduino_cli::get_or_download(worktree, &mut self.cached_arduino_cli_path) {
                Ok(cli_path) => {
                    let version = arduino_cli::extract_version(&cli_path)
                        .unwrap_or_else(|| "unknown".to_string());
                    self.installation_state
                        .record_arduino_cli_download(&version, cli_path.clone());
                    if let Err(e) = self.installation_state.save() {
                        eprintln!("Arduino: Failed to save installation state: {}", e);
                    }

                    if let Err(e) = setup::create_isolated_arduino_config(&self.installation_state)
                    {
                        eprintln!("Arduino: {}", e);
                    }

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

    // ============================================================================
    // Configuration Extraction
    // ============================================================================

    // Extract library paths from LSP settings
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

    // Validate FQBN format before use
    fn validate_and_use_fqbn<F>(&self, fqbn: &str, action: F)
    where
        F: FnOnce(&str),
    {
        if let Err(e) = arduino_cli::validate_fqbn(fqbn) {
            eprintln!("Arduino: {}", e);
        } else {
            action(fqbn);
        }
    }

    // ============================================================================
    // Automation
    // ============================================================================

    // Handle auto-install core and auto-generate compile_commands.json
    fn setup_automation(&mut self, args: &[String], worktree: &zed::Worktree) {
        let fqbn = utils::get_arg_value(args, "-fqbn").map(|s| s.to_string());

        // Auto-install core if enabled and FQBN is specified
        if utils::get_setting(worktree, "autoInstallCore", true) {
            if let Some(ref fqbn) = fqbn {
                self.validate_and_use_fqbn(fqbn, |fqbn| {
                    if let Some(core_id) = arduino_cli::extract_core_id(fqbn) {
                        if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                            if !arduino_cli::is_core_installed(cli_path, &core_id) {
                                let config_path = utils::get_arg_value(args, "-cli-config");
                                if arduino_cli::install_core(cli_path, &core_id, config_path)
                                    .is_ok()
                                {
                                    eprintln!("Arduino: Installed core {} automatically", core_id);
                                }
                            }
                        }
                    }
                });
            }
        }

        // Auto-generate compilation database if enabled
        if utils::get_setting(worktree, "autoGenerateCompileDb", true)
            && !sketches::check_compilation_database(worktree)
        {
            if let Some(ref fqbn) = fqbn {
                self.validate_and_use_fqbn(fqbn, |fqbn| {
                    if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                        let config_path = utils::get_arg_value(args, "-cli-config");
                        let library_paths = utils::get_library_paths(worktree);
                        if arduino_cli::generate_compile_db(
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

    /// Get FQBN with priority: user settings > detected board > last detected board
    fn get_or_detect_fqbn(&mut self, args: &[String], worktree: &zed::Worktree) -> Option<String> {
        // 1. Check if user specified FQBN in settings
        let user_fqbn = utils::get_string_setting(worktree, "fqbn", "");
        if !user_fqbn.is_empty() {
            return Some(user_fqbn);
        }

        // 2. Try to detect connected board
        if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
            if let Some((fqbn, port, name)) = arduino_cli::detect_connected_board(cli_path) {
                eprintln!(
                    "Arduino: Auto-detected board: {} (FQBN: {})",
                    name.as_deref().unwrap_or("Unknown"),
                    fqbn
                );
                if let Some(ref p) = port {
                    eprintln!("Arduino: Port: {}", p);
                }
                eprintln!("Arduino: Using detected FQBN (not saved to settings)");

                // Save to installation state for future use
                self.installation_state
                    .record_detected_board(fqbn.clone(), port, name);
                if let Err(e) = self.installation_state.save() {
                    eprintln!("Arduino: Failed to save detected board: {}", e);
                }

                return Some(fqbn);
            }
        }

        // 3. Use last detected board from installation state
        if let Some(last_fqbn) = self.installation_state.get_last_detected_fqbn() {
            eprintln!(
                "Arduino: No board detected, using last detected FQBN: {}",
                last_fqbn
            );
            return Some(last_fqbn.to_string());
        }

        // 4. No FQBN available
        None
    }
}

impl zed::Extension for ArduinoExtension {
    fn new() -> Self {
        Self {
            cached_language_server_path: None,
            cached_arduino_cli_path: None,
            cached_clangd_path: None,
            cached_clangd_info: None,
            cached_arduino_cli_info: None,
            installation_state: metadata::InstallationState::load(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Check for explicit sketch path override
        let explicit_sketch_path = utils::get_string_setting(worktree, "sketchPath", "");

        if !explicit_sketch_path.is_empty() {
            eprintln!(
                "Arduino: Using explicit sketch path from settings: {}",
                explicit_sketch_path
            );
        } else {
            // Detect Arduino sketches in the worktree
            let sketches = sketches::find_directories(worktree);
            if sketches.is_empty() {
                eprintln!("Arduino: Warning - No sketch directories found in workspace");
                eprintln!("Arduino: Looking for directories containing .ino or .pde files");
            } else if sketches.len() == 1 {
                let sketch_path = &sketches[0];
                if sketch_path == "." {
                    eprintln!("Arduino: Found sketch in workspace root");
                } else {
                    eprintln!("Arduino: Found sketch at: {}", sketch_path);
                }
            } else {
                eprintln!("Arduino: Found {} sketches in workspace:", sketches.len());
                for sketch in &sketches {
                    eprintln!("  - {}", sketch);
                }
                eprintln!(
                    "Arduino: Using first sketch (shallowest, then alphabetically): {}",
                    sketches[0]
                );
                eprintln!("Arduino: Note - Multiple sketches detected. For best results, open each sketch directory as a separate workspace.");
                eprintln!(
                    "Arduino: Tip - Set 'sketchPath' in settings to explicitly choose a sketch."
                );
            }
        }

        // Detect and record platform on first run
        if self.installation_state.get_platform().is_none() {
            let (platform, _) = zed::current_platform();
            let detected_platform = match platform {
                zed::Os::Linux => metadata::Platform::Linux,
                zed::Os::Mac => metadata::Platform::MacOS,
                zed::Os::Windows => metadata::Platform::Windows,
            };
            self.installation_state.record_platform(detected_platform);
            if let Err(e) = self.installation_state.save() {
                eprintln!("Arduino: Failed to save installation state: {}", e);
            }
        }

        // Auto-generate .zed/tasks.json if enabled (settings generation removed - use task instead)
        if let Err(e) = setup::auto_generate_tasks(worktree, &self.installation_state) {
            eprintln!("Arduino: {}", e);
        }

        // Check dependencies and report any issues
        setup::report_dependencies(worktree);

        // Get args and env from LSP settings
        let mut args: Vec<String> = Vec::new();
        let mut env: HashMap<String, String> = HashMap::new();

        if let Ok(lsp_settings) = LspSettings::for_worktree("arduino", worktree) {
            if let Some(binary) = lsp_settings.binary {
                // binary.arguments takes precedence
                if let Some(binary_args) = binary.arguments {
                    args = binary_args;
                }

                if let Some(binary_env) = binary.env {
                    env = binary_env;
                }
            }
        }

        // If no binary.arguments, fall back to ls.arguments from settings
        if args.is_empty() {
            args = utils::get_string_array_setting(worktree, "ls.arguments");
        }

        // Get the language server binary path
        let command_path = self.language_server_binary_path(language_server_id, worktree)?;

        // Add clangd path from settings if not already in args
        if !utils::has_arg(&args, "-clangd") {
            let clangd_path = utils::get_string_setting(worktree, "clangd.path", "");
            if !clangd_path.is_empty() {
                args.push("-clangd".to_string());
                args.push(clangd_path);
                self.add_clangd_arguments(&mut args, worktree);
            } else {
                // Fall back to auto-detection/download
                self.ensure_clangd_available(&mut args, worktree);
            }
        }

        // Add arduino-cli path from settings if not already in args
        if !utils::has_arg(&args, "-cli") {
            let cli_path = utils::get_string_setting(worktree, "cli.path", "");
            if !cli_path.is_empty() {
                args.push("-cli".to_string());
                args.push(cli_path);
            } else {
                // Fall back to auto-detection/download
                self.ensure_arduino_cli_available(&mut args, worktree)?;
            }
        }

        // Add FQBN from settings/detection if not already in args
        // NOTE: Must be after arduino-cli is resolved since detection needs it
        if !utils::has_arg(&args, "-fqbn") {
            if let Some(fqbn) = self.get_or_detect_fqbn(&args, worktree) {
                args.push("-fqbn".to_string());
                args.push(fqbn);
            } else {
                eprintln!("Arduino: Warning - FQBN not configured and no board detected");
                eprintln!(
                    "Arduino: Add 'fqbn' to lsp.arduino.settings or connect an Arduino board"
                );
            }
        }

        // Auto-detect or auto-create arduino-cli config
        let user_specified_cli_config = utils::has_arg(&args, "-cli-config");
        if !user_specified_cli_config {
            // First check settings
            let cli_config_setting = utils::get_string_setting(worktree, "cli.config", "");
            if !cli_config_setting.is_empty() {
                args.push("-cli-config".to_string());
                args.push(cli_config_setting);
            } else if self.installation_state.arduino_cli_uses_isolated_data() {
                // Use isolated config for downloaded arduino-cli
                let isolated_config = "arduino-cli-isolated.yaml";
                args.push("-cli-config".to_string());
                args.push(isolated_config.to_string());
            } else if let Some(config_path) =
                arduino_cli::find_config(worktree, utils::get_arg_value(&args, "-cli"))
            {
                // Use system config
                args.push("-cli-config".to_string());
                args.push(config_path);
            } else if utils::get_setting(worktree, "autoCreateConfig", true) {
                // Auto-create minimal config if enabled
                let config_path = format!("{}/.arduino-cli.yaml", worktree.root_path());

                // Check for additional board manager URLs
                let additional_urls = utils::get_string_array_setting(worktree, "additionalUrls");
                let urls_yaml = if additional_urls.is_empty() {
                    "board_manager:\n  additional_urls: []\n".to_string()
                } else {
                    let urls_list = additional_urls
                        .iter()
                        .map(|url| format!("    - {}", url))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("board_manager:\n  additional_urls:\n{}\n", urls_list)
                };

                if std::fs::write(&config_path, urls_yaml).is_ok() {
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

        // Run automation features
        self.setup_automation(&args, worktree);

        // Merge shell env with user-specified env vars (user settings override defaults)
        let default_env = match zed::current_platform().0 {
            zed::Os::Mac | zed::Os::Linux => worktree.shell_env(),
            zed::Os::Windows => Vec::new(),
        };
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
        // Provide workspace/configuration response from arduino LSP settings
        let settings = LspSettings::for_worktree("arduino", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();

        Ok(Some(settings))
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        _args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "arduino-board" => slash_commands::run_board_command(worktree),
            "arduino-config" => {
                slash_commands::run_config_command(worktree, &self.installation_state)
            }
            "arduino-sketch" => slash_commands::run_sketch_command(worktree),
            "arduino-cores" => slash_commands::run_cores_command(worktree),
            "arduino-libraries" => slash_commands::run_libraries_command(worktree),
            "arduino-errors" => slash_commands::run_errors_command(worktree),
            "arduino-examples" => slash_commands::run_examples_command(worktree),
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }

    fn complete_slash_command_argument(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        // Our commands don't take arguments, so no completions needed
        Ok(Vec::new())
    }
}

zed::register_extension!(ArduinoExtension);
