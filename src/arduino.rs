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
                    // Validate manually configured path too
                    if let Err(e) = language_server::validate(&path) {
                        eprintln!("Arduino: Warning - Configured language server: {}", e);
                    }
                    self.installation_state
                        .record_language_server_manual(path.clone());
                    let _ = self.installation_state.save();
                    return Ok(path.clone());
                }
            }
        }

        // Check for ls.path fallback in settings
        let ls_path = utils::get_string_setting(worktree, "ls.path", "");
        if !ls_path.is_empty() {
            // Validate manually configured path too
            if let Err(e) = language_server::validate(&ls_path) {
                eprintln!("Arduino: Warning - Configured language server: {}", e);
            }
            self.installation_state
                .record_language_server_manual(ls_path.clone());
            let _ = self.installation_state.save();
            return Ok(ls_path);
        }

        // Use language_server module to get binary (checks PATH, cache, then downloads)
        let path = language_server::get_or_download(
            language_server_id,
            worktree,
            &mut self.cached_language_server_path,
        )?;

        // Track downloaded version and validate it
        match language_server::validate(&path) {
            Ok(version_info) => {
                // Extract just the version number from the info string
                if let Some(version) = language_server::extract_version(&path) {
                    self.installation_state
                        .record_language_server_download(&version, path.clone());
                } else {
                    eprintln!(
                        "Arduino: Warning - Could not extract version: {}",
                        version_info
                    );
                    self.installation_state
                        .record_language_server_download("unknown", path.clone());
                }
            }
            Err(e) => {
                eprintln!("Arduino: Warning - Language server validation: {}", e);
                self.installation_state
                    .record_language_server_download("unknown", path.clone());
            }
        }
        let _ = self.installation_state.save();

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
            // Validate system tool too
            if let Err(e) = clangd::validate(&info.path) {
                eprintln!("Arduino: Warning - System clangd: {}", e);
            }
            self.installation_state
                .record_clangd_from_system(info.path.clone(), metadata::ToolSource::ZedManaged);
            let _ = self.installation_state.save();
            args.push("-clangd".to_string());
            args.push(info.path.clone());
            self.cached_clangd_info = Some(info);
            self.add_clangd_arguments(args, worktree);
        } else {
            match clangd::get_or_download(worktree, &mut self.cached_clangd_path) {
                Ok(clangd_path) => {
                    // Validate and extract version
                    match clangd::validate(&clangd_path) {
                        Ok(_version_info) => {
                            if let Some(version) = clangd::extract_version(&clangd_path) {
                                self.installation_state
                                    .record_clangd_download(&version, clangd_path.clone());
                            } else {
                                self.installation_state
                                    .record_clangd_download("unknown", clangd_path.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("Arduino: Warning - Clangd validation: {}", e);
                            self.installation_state
                                .record_clangd_download("unknown", clangd_path.clone());
                        }
                    }
                    let _ = self.installation_state.save();
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
            // Validate system tool too
            if let Err(e) = arduino_cli::validate(&info.path) {
                eprintln!("Arduino: Warning - System arduino-cli: {}", e);
            }
            self.installation_state
                .record_arduino_cli_from_path(info.path.clone());
            let _ = self.installation_state.save();
            args.push("-cli".to_string());
            args.push(info.path.clone());
            self.cached_arduino_cli_info = Some(info);
        } else {
            match arduino_cli::get_or_download(worktree, &mut self.cached_arduino_cli_path) {
                Ok(cli_path) => {
                    // Validate and extract version
                    match arduino_cli::validate(&cli_path) {
                        Ok(_version_info) => {
                            if let Some(version) = arduino_cli::extract_version(&cli_path) {
                                self.installation_state
                                    .record_arduino_cli_download(&version, cli_path.clone());
                            } else {
                                self.installation_state
                                    .record_arduino_cli_download("unknown", cli_path.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("Arduino: Warning - Arduino CLI validation: {}", e);
                            self.installation_state
                                .record_arduino_cli_download("unknown", cli_path.clone());
                        }
                    }
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

    /// Check if compile database should be generated
    /// Only generate when:
    /// - An Arduino sketch exists in the workspace
    /// - Database doesn't exist
    /// - FQBN has changed since last generation
    /// - User hasn't disabled auto-generation
    fn should_generate_compile_database(
        &self,
        worktree: &zed::Worktree,
        fqbn: &Option<String>,
    ) -> bool {
        // Check if user disabled auto-generation
        if !utils::get_setting(worktree, "autoGenerateCompileDb", true) {
            return false;
        }

        // Check if there's actually an Arduino sketch in the workspace
        let detected_sketches = sketches::find_directories(worktree);
        if detected_sketches.is_empty() {
            // No Arduino sketch found - don't generate
            return false;
        }

        // If database exists, check if FQBN changed
        if sketches::check_compilation_database(worktree) {
            if let Some(current_fqbn) = fqbn {
                if let Some(cached_fqbn) = self.installation_state.get_compile_db_fqbn() {
                    // Skip if same FQBN
                    if current_fqbn == cached_fqbn {
                        return false;
                    }
                }
            }
        }

        // Generate if database missing or FQBN changed
        true
    }

    // Handle auto-install core and auto-generate compile_commands.json
    fn setup_automation(&mut self, args: &[String], worktree: &zed::Worktree) {
        let fqbn = utils::get_arg_value(args, "-fqbn").map(|s| s.to_string());

        // Auto-install core if enabled and FQBN is specified
        // Setup automation: auto-install cores
        if utils::get_setting(worktree, "autoInstallCore", true) {
            if let Some(ref fqbn) = fqbn {
                self.validate_and_use_fqbn(fqbn, |fqbn| {
                    if let Some(core_id) = arduino_cli::extract_core_id(fqbn) {
                        if !core_id.is_empty() {
                            if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                                if !arduino_cli::is_core_installed(cli_path, &core_id) {
                                    let config_path = utils::get_arg_value(args, "-cli-config");
                                    if arduino_cli::install_core(cli_path, &core_id, config_path)
                                        .is_ok()
                                    {
                                        eprintln!(
                                            "Arduino: Installed core {} automatically",
                                            core_id
                                        );
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }

        // Smart compile database generation: only when needed for IntelliSense
        if self.should_generate_compile_database(worktree, &fqbn) {
            if let Some(ref fqbn) = fqbn {
                // Validate FQBN first
                if let Err(e) = arduino_cli::validate_fqbn(fqbn) {
                    eprintln!("Arduino: {}", e);
                } else if let Some(cli_path) = utils::get_arg_value(args, "-cli") {
                    // Determine sketch path (same logic as language_server_command)
                    let detected_sketches = sketches::find_directories(worktree);
                    let explicit_sketch_path =
                        utils::get_string_setting(worktree, "sketchPath", "");

                    let sketch_path = if !explicit_sketch_path.is_empty() {
                        format!("{}/{}", worktree.root_path(), explicit_sketch_path)
                    } else if !detected_sketches.is_empty() {
                        let relative = &detected_sketches[0];
                        if relative == "." {
                            worktree.root_path()
                        } else {
                            format!("{}/{}", worktree.root_path(), relative)
                        }
                    } else {
                        worktree.root_path()
                    };

                    let config_path = utils::get_arg_value(args, "-cli-config");
                    let library_paths = utils::get_library_paths(worktree);

                    eprintln!(
                        "Arduino: Generating compile_commands.json for IntelliSense (10-30s)..."
                    );

                    match arduino_cli::generate_compile_db(
                        cli_path,
                        fqbn,
                        config_path,
                        &library_paths,
                        &sketch_path,
                        worktree,
                    ) {
                        Ok(_) => {
                            eprintln!("Arduino: ✓ Compilation database generated successfully");
                            // Cache the FQBN to avoid regeneration
                            self.installation_state
                                .record_compile_db_fqbn(fqbn.to_string());
                            let _ = self.installation_state.save();
                        }
                        Err(e) => {
                            eprintln!(
                                "Arduino: Warning - Failed to generate compilation database: {}",
                                e
                            );
                            eprintln!(
                                "Arduino: IntelliSense will be limited. Run a compile task to generate it."
                            );
                        }
                    }
                }
            } else {
                eprintln!("Arduino: Compilation database not generated - FQBN not configured");
                eprintln!(
                    "Arduino: Set 'fqbn' in settings or connect a board for full IntelliSense"
                );
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
                    "Arduino: Auto-detected board: {} ({})",
                    name.as_deref().unwrap_or("Unknown"),
                    fqbn
                );

                // Save to installation state for future use
                self.installation_state
                    .record_detected_board(fqbn.clone(), port, name);
                let _ = self.installation_state.save();

                return Some(fqbn);
            }
        }

        // 3. Use last detected board from installation state
        if let Some(last_fqbn) = self.installation_state.get_last_detected_fqbn() {
            eprintln!("Arduino: Using previously detected FQBN: {}", last_fqbn);
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
        // Determine sketch path: auto-detect, then settings, then default to workspace root
        let sketch_path = {
            // First, try to auto-detect sketch locations
            let detected_sketches = sketches::find_directories(worktree);

            // Check if user explicitly set sketchPath in settings
            let explicit_sketch_path = utils::get_string_setting(worktree, "sketchPath", "");

            if !explicit_sketch_path.is_empty() {
                explicit_sketch_path
            } else if !detected_sketches.is_empty() {
                // Use the first detected sketch (shallowest directory)
                let detected = detected_sketches[0].clone();
                if detected_sketches.len() > 1 {
                    eprintln!(
                        "Arduino: Found {} sketches, using: {} (set 'sketchPath' to override)",
                        detected_sketches.len(),
                        detected
                    );
                }
                detected
            } else {
                eprintln!("Arduino: No sketch detected, using workspace root");
                eprintln!(
                    "Arduino: Tip: Set 'sketchPath' in settings if sketch is in a subdirectory"
                );
                ".".to_string()
            }
        };

        // Detect and record platform on first run
        if self.installation_state.get_platform().is_none() {
            let (platform, _) = zed::current_platform();
            let detected_platform = match platform {
                zed::Os::Linux => metadata::Platform::Linux,
                zed::Os::Mac => metadata::Platform::MacOS,
                zed::Os::Windows => metadata::Platform::Windows,
            };
            self.installation_state.record_platform(detected_platform);
            let _ = self.installation_state.save();
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

        // Add sketch path argument if not already in args
        if !utils::has_arg(&args, "-sketch-path") {
            // Convert relative path to absolute
            let absolute_sketch_path = if sketch_path == "." {
                worktree.root_path()
            } else {
                format!("{}/{}", worktree.root_path(), sketch_path)
            };

            args.push("-sketch-path".to_string());
            args.push(absolute_sketch_path.clone());
        }

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
                eprintln!("Arduino: Add 'fqbn' to settings or connect an Arduino board");
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
