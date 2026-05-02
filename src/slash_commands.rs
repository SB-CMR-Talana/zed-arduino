//! Slash command handlers for Arduino extension AI Assistant integration.

use zed_extension_api::{self as zed, serde_json};

use crate::metadata::InstallationState;
use crate::sketches;
use crate::utils;

// ============================================================================
// Command: /arduino-board
// ============================================================================

/// Displays current board configuration (FQBN, port, detected info)
pub fn run_board_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();
    let start_pos = output.len();

    output.push_str("# Arduino Board Configuration\n\n");

    // Get FQBN setting
    let fqbn = utils::get_string_setting(worktree, "fqbn", "");
    if !fqbn.is_empty() {
        output.push_str(&format!("**FQBN:** `{}`\n", fqbn));

        // Parse FQBN for board details
        let parts: Vec<&str> = fqbn.split(':').collect();
        if parts.len() >= 3 {
            output.push_str(&format!("  - Vendor: `{}`\n", parts[0]));
            output.push_str(&format!("  - Architecture: `{}`\n", parts[1]));
            output.push_str(&format!("  - Board: `{}`\n", parts[2]));
        }
    } else {
        output.push_str("**FQBN:** Not configured (will use auto-detection)\n");
    }

    output.push_str("\n");

    // Get port settings
    let port = utils::get_string_setting(worktree, "cli.port", "");
    if !port.is_empty() {
        output.push_str(&format!("**Port:** `{}`\n", port));
    } else {
        output.push_str("**Port:** Not configured (will use auto-detection)\n");
    }

    let baud_rate = utils::get_string_setting(worktree, "cli.baudRate", "9600");
    output.push_str(&format!("**Baud Rate:** `{}`\n", baud_rate));

    output.push_str("\n");

    // Board manager URLs
    let urls = utils::get_string_setting(worktree, "cli.additionalUrls", "");
    if !urls.is_empty() {
        output.push_str("**Additional Board URLs:**\n");
        for url in urls.split(',') {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                output.push_str(&format!("  - `{}`\n", trimmed));
            }
        }
    } else {
        output.push_str("**Additional Board URLs:** None configured\n");
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: start_pos as u32,
            end: output.len() as u32,
        },
        label: "Board Configuration".to_string(),
    });

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

// ============================================================================
// Command: /arduino-config
// ============================================================================

/// Displays extension configuration (settings, paths, tool versions)
pub fn run_config_command(
    worktree: Option<&zed::Worktree>,
    installation_state: &InstallationState,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    // Header
    output.push_str("# Arduino Extension Configuration\n\n");

    // ========== Tool Paths & Versions ==========
    let section_start = output.len();
    output.push_str("## Tool Paths & Versions\n\n");

    // Arduino CLI
    let cli_path = utils::get_string_setting(worktree, "cli.path", "");
    if !cli_path.is_empty() {
        output.push_str(&format!("**Arduino CLI Path:** `{}`\n", cli_path));
    } else {
        output.push_str("**Arduino CLI Path:** Auto-detected or downloaded\n");
    }
    if let Some(ref metadata) = installation_state.arduino_cli {
        output.push_str(&format!(
            "  - Version: `{}`\n",
            metadata.version.as_deref().unwrap_or("unknown")
        ));
        output.push_str(&format!("  - Location: `{}`\n", metadata.location));
    }
    output.push_str("\n");

    // Clangd
    let clangd_path = utils::get_string_setting(worktree, "clangd.path", "");
    if !clangd_path.is_empty() {
        output.push_str(&format!("**Clangd Path:** `{}`\n", clangd_path));
    } else {
        output.push_str("**Clangd Path:** Auto-detected or downloaded\n");
    }
    if let Some(ref metadata) = installation_state.clangd {
        output.push_str(&format!(
            "  - Version: `{}`\n",
            metadata.version.as_deref().unwrap_or("unknown")
        ));
        output.push_str(&format!("  - Location: `{}`\n", metadata.location));
    }
    let clangd_args = utils::get_string_setting(worktree, "clangd.arguments", "");
    if !clangd_args.is_empty() {
        output.push_str(&format!("  - Custom Args: `{}`\n", clangd_args));
    }
    output.push_str("\n");

    // Language Server
    let ls_path = utils::get_string_setting(worktree, "ls.path", "");
    if !ls_path.is_empty() {
        output.push_str(&format!("**Language Server Path:** `{}`\n", ls_path));
    } else {
        output.push_str("**Language Server Path:** Auto-detected or downloaded\n");
    }
    if let Some(ref metadata) = installation_state.arduino_language_server {
        output.push_str(&format!(
            "  - Version: `{}`\n",
            metadata.version.as_deref().unwrap_or("unknown")
        ));
        output.push_str(&format!("  - Location: `{}`\n", metadata.location));
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Tool Paths & Versions".to_string(),
    });

    // ========== Arduino CLI Settings ==========
    let section_start = output.len();
    output.push_str("\n## Arduino CLI Settings\n\n");

    let config_path = utils::get_string_setting(worktree, "cli.config", "");
    if !config_path.is_empty() {
        output.push_str(&format!("**Config File:** `{}`\n", config_path));
    } else {
        output.push_str("**Config File:** Using default location\n");
    }

    let data_dir = utils::get_string_setting(worktree, "cli.dataDir", "");
    if !data_dir.is_empty() {
        output.push_str(&format!("**Data Directory:** `{}`\n", data_dir));
    } else {
        output.push_str("**Data Directory:** Using default location\n");
    }

    let user_dir = utils::get_string_setting(worktree, "cli.userDir", "");
    if !user_dir.is_empty() {
        output.push_str(&format!("**User Directory:** `{}`\n", user_dir));
    } else {
        output.push_str("**User Directory:** Using default location\n");
    }

    output.push_str("\n");

    // Compile arguments
    if let Ok(lsp_settings) = zed::settings::LspSettings::for_worktree("arduino", worktree) {
        if let Some(settings_value) = lsp_settings.settings {
            if let Some(cli) = settings_value.get("cli") {
                if let Some(compile_args) = cli.get("compileArguments") {
                    if let Some(args_array) = compile_args.as_array() {
                        if !args_array.is_empty() {
                            output.push_str("**Compile Arguments:**\n");
                            for arg in args_array {
                                if let Some(s) = arg.as_str() {
                                    output.push_str(&format!("  - `{}`\n", s));
                                }
                            }
                            output.push_str("\n");
                        }
                    }
                }
            }
        }
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Arduino CLI Settings".to_string(),
    });

    // ========== Automation Settings ==========
    let section_start = output.len();
    output.push_str("\n## Automation Settings\n\n");

    let auto_install = utils::get_setting(worktree, "autoInstallTools", true);
    output.push_str(&format!(
        "**Auto Install Tools:** {}\n",
        if auto_install {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    ));

    let auto_setup = utils::get_setting(worktree, "autoSetup", true);
    output.push_str(&format!(
        "**Auto Setup Projects:** {}\n",
        if auto_setup {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    ));

    let auto_compile_db = utils::get_setting(worktree, "autoGenerateCompileDb", true);
    output.push_str(&format!(
        "**Auto Generate compile_commands.json:** {}\n",
        if auto_compile_db {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    ));

    let auto_board_detect = utils::get_setting(worktree, "autoDetectBoard", true);
    output.push_str(&format!(
        "**Auto Detect Board:** {}\n",
        if auto_board_detect {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        }
    ));

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Automation Settings".to_string(),
    });

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

// ============================================================================
// Command: /arduino-sketch
// ============================================================================

/// Displays sketch structure (.ino files, libraries, compile database)
pub fn run_sketch_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    output.push_str("# Arduino Sketch Structure\n\n");

    // ========== Sketch Detection ==========
    let section_start = output.len();
    output.push_str("## Sketch Files\n\n");

    let explicit_sketch = utils::get_string_setting(worktree, "sketchPath", "");
    if !explicit_sketch.is_empty() {
        output.push_str(&format!(
            "**Configured Sketch Path:** `{}`\n\n",
            explicit_sketch
        ));
    }

    let sketches = sketches::find_directories(worktree);
    if sketches.is_empty() {
        output
            .push_str("**Status:** ⚠️ No Arduino sketch files (.ino or .pde) found in workspace\n");
    } else if sketches.len() == 1 {
        output.push_str(&format!("**Status:** ✓ Found 1 sketch directory\n"));
        output.push_str(&format!("**Location:** `{}`\n", sketches[0]));
    } else {
        output.push_str(&format!(
            "**Status:** ⚠️ Multiple sketch directories found ({})\n\n",
            sketches.len()
        ));
        output.push_str(
            "**Note:** The extension uses the first sketch (by depth, then alphabetically).\n",
        );
        output.push_str("Consider using `sketchPath` setting to specify which sketch to use.\n\n");
        output.push_str("**Detected Sketches:**\n");
        for (i, sketch_path) in sketches.iter().enumerate() {
            let marker = if i == 0 { "→" } else { " " };
            output.push_str(&format!("  {} `{}`\n", marker, sketch_path));
        }
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Sketch Files".to_string(),
    });

    // ========== Libraries ==========
    let section_start = output.len();
    output.push_str("\n## Library Configuration\n\n");

    let library_paths = utils::get_string_setting(worktree, "libraryPaths", "");
    if !library_paths.is_empty() {
        output.push_str("**Custom Library Paths:**\n");
        for path in library_paths.split(',') {
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                output.push_str(&format!("  - `{}`\n", trimmed));
            }
        }
    } else {
        output.push_str("**Custom Library Paths:** None configured\n");
        output.push_str("  (Using Arduino CLI default library locations)\n");
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Library Configuration".to_string(),
    });

    // ========== Compile Database ==========
    let section_start = output.len();
    output.push_str("\n## Compilation Database\n\n");

    let compile_db_path = utils::get_string_setting(worktree, "compileDb.path", "");
    if !compile_db_path.is_empty() {
        output.push_str(&format!("**Custom Path:** `{}`\n", compile_db_path));
    } else {
        output.push_str("**Path:** Auto-detected in sketch directory\n");
    }

    // Check if compile_commands.json exists
    if !sketches.is_empty() {
        let sketch_dir = &sketches[0];
        let worktree_root = worktree.root_path();
        let compile_db_path = if sketch_dir == "." {
            format!("{}/compile_commands.json", worktree_root)
        } else {
            format!("{}/{}/compile_commands.json", worktree_root, sketch_dir)
        };

        match std::fs::metadata(&compile_db_path) {
            Ok(_) => {
                output.push_str("**Status:** ✓ `compile_commands.json` found\n");
            }
            Err(_) => {
                output.push_str("**Status:** ⚠️ `compile_commands.json` not found\n");
                output.push_str("  (Will be generated automatically on first compile)\n");
            }
        }
    }

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Compilation Database".to_string(),
    });

    // ========== Workspace Info ==========
    let section_start = output.len();
    output.push_str("\n## Workspace Information\n\n");

    output.push_str(&format!("**Root Path:** `{}`\n", worktree.root_path()));

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: section_start as u32,
            end: output.len() as u32,
        },
        label: "Workspace Information".to_string(),
    });

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

// ============================================================================
// Command: /arduino-cores
// ============================================================================

/// Lists installed Arduino board cores with versions
pub fn run_cores_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    output.push_str("# Arduino Board Cores\n\n");

    // Get arduino-cli path
    let cli_path = utils::get_string_setting(worktree, "cli.path", "");
    let cli_command = if !cli_path.is_empty() {
        cli_path
    } else {
        "arduino-cli".to_string()
    };

    // Run arduino-cli core list
    let result = std::process::Command::new(&cli_command)
        .arg("core")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output();

    match result {
        Ok(cmd_output) => {
            if cmd_output.status.success() {
                let section_start = output.len();
                output.push_str("## Installed Cores\n\n");

                let stdout = String::from_utf8_lossy(&cmd_output.stdout);

                // Try to parse JSON output
                if let Ok(cores_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(cores_array) = cores_json.as_array() {
                        if cores_array.is_empty() {
                            output.push_str("⚠️ **No cores installed**\n\n");
                            output.push_str("Install a core using:\n");
                            output.push_str("```\n");
                            output.push_str("arduino-cli core install <core_name>\n");
                            output.push_str("```\n\n");
                            output.push_str("Popular cores:\n");
                            output.push_str(
                                "  - `arduino:avr` - Arduino AVR Boards (Uno, Nano, Mega)\n",
                            );
                            output
                                .push_str("  - `arduino:samd` - Arduino SAMD Boards (Zero, MKR)\n");
                            output.push_str("  - `esp32:esp32` - ESP32 Boards\n");
                            output.push_str("  - `esp8266:esp8266` - ESP8266 Boards\n");
                        } else {
                            output.push_str(&format!(
                                "✓ **Found {} installed core(s)**\n\n",
                                cores_array.len()
                            ));

                            for core in cores_array {
                                if let Some(id) = core.get("id").and_then(|v| v.as_str()) {
                                    let version = core
                                        .get("version")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let name =
                                        core.get("name").and_then(|v| v.as_str()).unwrap_or(id);

                                    output.push_str(&format!("### `{}`\n\n", id));
                                    output.push_str(&format!("**Name:** {}\n", name));
                                    output.push_str(&format!("**Version:** `{}`\n", version));

                                    // List boards if available
                                    if let Some(boards) =
                                        core.get("boards").and_then(|v| v.as_array())
                                    {
                                        if !boards.is_empty() {
                                            output.push_str(&format!(
                                                "**Boards:** {} available\n",
                                                boards.len()
                                            ));

                                            // Show first few boards as examples
                                            let show_count = std::cmp::min(5, boards.len());
                                            for board in boards.iter().take(show_count) {
                                                if let Some(fqbn) =
                                                    board.get("fqbn").and_then(|v| v.as_str())
                                                {
                                                    let board_name = board
                                                        .get("name")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or(fqbn);
                                                    output.push_str(&format!(
                                                        "  - {} (`{}`)\n",
                                                        board_name, fqbn
                                                    ));
                                                }
                                            }
                                            if boards.len() > show_count {
                                                output.push_str(&format!(
                                                    "  - ... and {} more\n",
                                                    boards.len() - show_count
                                                ));
                                            }
                                        }
                                    }
                                    output.push_str("\n");
                                }
                            }
                        }
                    }
                } else {
                    // Fallback to text parsing if JSON fails
                    let lines: Vec<&str> = stdout.lines().collect();
                    if lines.len() <= 1 {
                        output.push_str("⚠️ **No cores installed**\n\n");
                        output.push_str(
                            "Install a core using: `arduino-cli core install <core_name>`\n",
                        );
                    } else {
                        output.push_str(&format!(
                            "✓ **Found {} installed core(s)**\n\n",
                            lines.len() - 1
                        ));
                        output.push_str("```\n");
                        output.push_str(&stdout);
                        output.push_str("\n```\n");
                    }
                }

                sections.push(zed::SlashCommandOutputSection {
                    range: zed::Range {
                        start: section_start as u32,
                        end: output.len() as u32,
                    },
                    label: "Installed Cores".to_string(),
                });
            } else {
                let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                output.push_str(&format!(
                    "❌ **Failed to list cores**\n\n```\n{}\n```\n",
                    stderr
                ));
            }
        }
        Err(e) => {
            output.push_str(&format!(
                "❌ **Failed to run arduino-cli**\n\nError: {}\n\n",
                e
            ));
            output.push_str("**Possible causes:**\n");
            output.push_str("  - Arduino CLI is not installed or not in PATH\n");
            output.push_str("  - Check `cli.path` setting if using custom location\n");
        }
    }

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

// ============================================================================
// Command: /arduino-libraries
// ============================================================================

/// Lists installed Arduino libraries with versions and locations
pub fn run_libraries_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    output.push_str("# Arduino Libraries\n\n");

    // === Library Search Paths Section ===
    let paths_section_start = output.len();
    output.push_str("## Library Search Paths\n\n");
    output.push_str(
        "Arduino searches these directories for libraries when compiling (in priority order):\n\n",
    );

    // Custom library paths from settings
    let custom_paths = utils::get_library_paths(worktree);
    if !custom_paths.is_empty() {
        output.push_str("**Custom paths (from settings):**\n");
        for path in &custom_paths {
            output.push_str(&format!("  - `{}` ✓\n", path));
        }
        output.push_str("\n");
    }

    // System library paths
    output.push_str("**System paths:**\n");

    // Get home directory for path construction
    if let Some(home) = utils::get_home(worktree) {
        output.push_str(&format!(
            "  - `{}/Arduino/libraries/` (User libraries)\n",
            home
        ));
        output.push_str(&format!(
            "  - `{}/.arduino15/libraries/` (Arduino IDE libraries)\n",
            home
        ));
        output.push_str(&format!(
            "  - `{}/.arduino15/packages/*/hardware/*/libraries/` (Core libraries)\n",
            home
        ));
    } else {
        output.push_str("  - `~/Arduino/libraries/` (User libraries)\n");
        output.push_str("  - `~/.arduino15/libraries/` (Arduino IDE libraries)\n");
        output.push_str("  - `~/.arduino15/packages/*/hardware/*/libraries/` (Core libraries)\n");
    }
    output.push_str("\n");
    output.push_str("**Note:** Custom libraries and headers in these directories are available to your sketch.\n\n");

    sections.push(zed::SlashCommandOutputSection {
        range: zed::Range {
            start: paths_section_start as u32,
            end: output.len() as u32,
        },
        label: "Library Search Paths".to_string(),
    });

    // Get arduino-cli path
    let cli_path = utils::get_string_setting(worktree, "cli.path", "");
    let cli_command = if !cli_path.is_empty() {
        cli_path
    } else {
        "arduino-cli".to_string()
    };

    // Run arduino-cli lib list
    let result = std::process::Command::new(&cli_command)
        .arg("lib")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output();

    match result {
        Ok(cmd_output) => {
            if cmd_output.status.success() {
                let section_start = output.len();
                output.push_str("## Installed Libraries\n\n");

                let stdout = String::from_utf8_lossy(&cmd_output.stdout);

                // Try to parse JSON output
                if let Ok(libs_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(libs_array) = libs_json
                        .get("installed_libraries")
                        .and_then(|v| v.as_array())
                    {
                        if libs_array.is_empty() {
                            output.push_str("⚠️ **No libraries installed**\n\n");
                            output.push_str("Install a library using:\n");
                            output.push_str("```\n");
                            output.push_str("arduino-cli lib install <library_name>\n");
                            output.push_str("```\n\n");
                            output.push_str("Search for libraries:\n");
                            output.push_str("```\n");
                            output.push_str("arduino-cli lib search <keyword>\n");
                            output.push_str("```\n");
                        } else {
                            output.push_str(&format!(
                                "✓ **Found {} installed library/libraries**\n\n",
                                libs_array.len()
                            ));

                            // Group by location type
                            let mut user_libs = Vec::new();
                            let mut builtin_libs = Vec::new();

                            for lib in libs_array {
                                let location = lib
                                    .get("location")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");

                                if location.to_lowercase().contains("builtin")
                                    || location.to_lowercase().contains("ide")
                                {
                                    builtin_libs.push(lib);
                                } else {
                                    user_libs.push(lib);
                                }
                            }

                            // Display user libraries first
                            if !user_libs.is_empty() {
                                output.push_str(&format!(
                                    "### User Libraries ({})\n\n",
                                    user_libs.len()
                                ));
                                for lib in user_libs {
                                    if let Some(name) = lib
                                        .get("library")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                    {
                                        let version = lib
                                            .get("library")
                                            .and_then(|v| v.get("version"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");
                                        let location = lib
                                            .get("install_dir")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");

                                        output.push_str(&format!("**{}** `v{}`\n", name, version));

                                        if let Some(desc) = lib
                                            .get("library")
                                            .and_then(|v| v.get("sentence"))
                                            .and_then(|v| v.as_str())
                                        {
                                            if !desc.is_empty() {
                                                output.push_str(&format!("  - {}", desc));
                                                if !desc.ends_with('.') {
                                                    output.push_str(".\n");
                                                } else {
                                                    output.push_str("\n");
                                                }
                                            }
                                        }

                                        if !location.is_empty() {
                                            // Show shortened path
                                            let shortened = if location.len() > 60 {
                                                format!("...{}", &location[location.len() - 57..])
                                            } else {
                                                location.to_string()
                                            };
                                            output.push_str(&format!(
                                                "  - Location: `{}`\n",
                                                shortened
                                            ));
                                        }
                                        output.push_str("\n");
                                    }
                                }
                            }

                            // Display built-in libraries
                            if !builtin_libs.is_empty() {
                                output.push_str(&format!(
                                    "### Built-in Libraries ({})\n\n",
                                    builtin_libs.len()
                                ));
                                for lib in builtin_libs {
                                    if let Some(name) = lib
                                        .get("library")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                    {
                                        let version = lib
                                            .get("library")
                                            .and_then(|v| v.get("version"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");

                                        output.push_str(&format!("**{}** `v{}`\n", name, version));

                                        if let Some(desc) = lib
                                            .get("library")
                                            .and_then(|v| v.get("sentence"))
                                            .and_then(|v| v.as_str())
                                        {
                                            if !desc.is_empty() {
                                                output.push_str(&format!("  - {}", desc));
                                                if !desc.ends_with('.') {
                                                    output.push_str(".\n");
                                                } else {
                                                    output.push_str("\n");
                                                }
                                            }
                                        }
                                        output.push_str("\n");
                                    }
                                }
                            }
                        }
                    } else {
                        output.push_str("⚠️ **No libraries found**\n");
                    }
                } else {
                    // Fallback to text parsing if JSON fails
                    let lines: Vec<&str> = stdout.lines().collect();
                    if lines.len() <= 1 {
                        output.push_str("⚠️ **No libraries installed**\n\n");
                        output.push_str(
                            "Install a library using: `arduino-cli lib install <library_name>`\n",
                        );
                    } else {
                        output.push_str(&format!(
                            "✓ **Found {} installed library/libraries**\n\n",
                            lines.len() - 1
                        ));
                        output.push_str("```\n");
                        output.push_str(&stdout);
                        output.push_str("\n```\n");
                    }
                }

                sections.push(zed::SlashCommandOutputSection {
                    range: zed::Range {
                        start: section_start as u32,
                        end: output.len() as u32,
                    },
                    label: "Installed Libraries".to_string(),
                });
            } else {
                let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                output.push_str(&format!(
                    "❌ **Failed to list libraries**\n\n```\n{}\n```\n",
                    stderr
                ));
            }
        }
        Err(e) => {
            output.push_str(&format!(
                "❌ **Failed to run arduino-cli**\n\nError: {}\n\n",
                e
            ));
            output.push_str("**Possible causes:**\n");
            output.push_str("  - Arduino CLI is not installed or not in PATH\n");
            output.push_str("  - Check `cli.path` setting if using custom location\n");
        }
    }

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

// ============================================================================
// Command: /arduino-errors
// ============================================================================

/// Shows last compilation errors from .zed/last_compile.log
pub fn run_errors_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    output.push_str("# Arduino Compilation Errors\n\n");

    let worktree_root = worktree.root_path();
    let log_file = format!("{}/.zed/last_compile.log", worktree_root);

    // Check if log file exists
    if let Ok(contents) = std::fs::read_to_string(&log_file) {
        let section_start = output.len();

        // Parse the compilation output
        let (has_errors, errors, warnings, success_msg) = parse_compilation_output(&contents);

        if has_errors {
            output.push_str("## ❌ Compilation Failed\n\n");

            if !errors.is_empty() {
                output.push_str("### Errors\n\n");
                for error in &errors {
                    output.push_str(&format!("```\n{}\n```\n\n", error));
                }
            }

            if !warnings.is_empty() {
                output.push_str("### Warnings\n\n");
                for warning in &warnings {
                    output.push_str(&format!("- {}\n", warning));
                }
                output.push_str("\n");
            }
        } else if !warnings.is_empty() {
            output.push_str("## ✓ Compilation Succeeded (with warnings)\n\n");
            output.push_str("### Warnings\n\n");
            for warning in &warnings {
                output.push_str(&format!("- {}\n", warning));
            }
            output.push_str("\n");
        } else {
            output.push_str("## ✓ Compilation Succeeded\n\n");
            if let Some(msg) = success_msg {
                output.push_str(&format!("{}\n\n", msg));
            }
        }

        sections.push(zed::SlashCommandOutputSection {
            range: zed::Range {
                start: section_start as u32,
                end: output.len() as u32,
            },
            label: "Compilation Result".to_string(),
        });

        // Add full log as collapsible section
        let section_start = output.len();
        output.push_str("## Full Compilation Output\n\n");
        output.push_str("```\n");
        // Limit to last 3000 characters to avoid huge outputs
        if contents.len() > 3000 {
            output.push_str("... (showing last 3000 characters) ...\n\n");
            output.push_str(&contents[contents.len() - 3000..]);
        } else {
            output.push_str(&contents);
        }
        output.push_str("\n```\n");

        sections.push(zed::SlashCommandOutputSection {
            range: zed::Range {
                start: section_start as u32,
                end: output.len() as u32,
            },
            label: "Full Output".to_string(),
        });
    } else {
        output.push_str("⚠️ **No compilation log found**\n\n");
        output.push_str(&format!("**Expected location:** `{}`\n\n", log_file));
        output.push_str("No recent compilation found. Compilation output is automatically saved when you run:\n\n");
        output.push_str("- **Arduino: Compile** task\n");
        output.push_str("- **Arduino: Compile, Upload & Monitor** task\n");
        output.push_str("- **Arduino: Compile & Upload** task\n\n");
        output.push_str("**To compile your sketch:**\n");
        output.push_str("1. Open command palette (Cmd/Ctrl + Shift + P)\n");
        output.push_str("2. Select `task: spawn`\n");
        output.push_str("3. Choose `Arduino: Compile`\n");
    }

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}

/// Parse arduino-cli compilation output to extract errors and warnings
fn parse_compilation_output(contents: &str) -> (bool, Vec<String>, Vec<String>, Option<String>) {
    let mut has_errors = false;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut success_msg = None;

    let lines: Vec<&str> = contents.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check for error indicators
        if line.contains("error:") {
            has_errors = true;
            let mut error_block = String::new();

            // Capture the error line and context
            error_block.push_str(line);
            error_block.push('\n');

            // Capture following lines that are part of the error (indented or continuation)
            i += 1;
            while i < lines.len() {
                let next_line = lines[i];
                // Stop at next error/warning or blank line followed by non-indented text
                if next_line.contains("error:") || next_line.contains("warning:") {
                    i -= 1; // Back up so outer loop processes this line
                    break;
                }
                if next_line.trim().is_empty() {
                    if i + 1 < lines.len()
                        && !lines[i + 1].starts_with(' ')
                        && !lines[i + 1].trim().is_empty()
                    {
                        break;
                    }
                }
                error_block.push_str(next_line);
                error_block.push('\n');
                i += 1;

                // Limit error block size
                if error_block.len() > 500 {
                    break;
                }
            }

            errors.push(error_block.trim_end().to_string());
        }
        // Check for warning indicators
        else if line.contains("warning:") {
            // Extract just the warning message
            warnings.push(line.trim().to_string());
        }
        // Check for success messages
        else if line.contains("Sketch uses") || line.contains("Global variables use") {
            if success_msg.is_none() {
                success_msg = Some(String::new());
            }
            if let Some(ref mut msg) = success_msg {
                msg.push_str(line.trim());
                msg.push('\n');
            }
        }

        i += 1;
    }

    (has_errors, errors, warnings, success_msg)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compilation_output_success() {
        let output = "Sketch uses 924 bytes (2%) of program storage space. Maximum is 32256 bytes.\nGlobal variables use 9 bytes (0%) of dynamic memory, leaving 2039 bytes for local variables. Maximum is 2048 bytes.";

        let (has_errors, errors, warnings, success_msg) = parse_compilation_output(output);

        assert!(!has_errors);
        assert!(errors.is_empty());
        assert!(warnings.is_empty());
        assert!(success_msg.is_some());
        assert!(success_msg.unwrap().contains("924 bytes"));
    }

    #[test]
    fn test_parse_compilation_output_with_error() {
        let output = "sketch.ino:5:1: error: 'digitalWrit' was not declared in this scope\n  digitalWrit(LED_BUILTIN, HIGH);\n  ^~~~~~~~~~~\nsketch.ino:5:1: note: suggested alternative: 'digitalWrite'\n  digitalWrit(LED_BUILTIN, HIGH);\n  ^~~~~~~~~~~\n  digitalWrite";

        let (has_errors, errors, warnings, _success_msg) = parse_compilation_output(output);

        assert!(has_errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("digitalWrit"));
        assert!(errors[0].contains("was not declared"));
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_compilation_output_with_warning() {
        let output = "sketch.ino:3:5: warning: unused variable 'x' [-Wunused-variable]\nSketch uses 924 bytes (2%) of program storage space. Maximum is 32256 bytes.";

        let (has_errors, errors, warnings, success_msg) = parse_compilation_output(output);

        assert!(!has_errors);
        assert!(errors.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unused variable"));
        assert!(success_msg.is_some());
    }

    #[test]
    fn test_parse_compilation_output_multiple_errors() {
        let output = "sketch.ino:5:1: error: 'digitalWrit' was not declared\nsketch.ino:10:1: error: expected ';' before 'delay'\ncompilation terminated.";

        let (has_errors, errors, _warnings, _success_msg) = parse_compilation_output(output);

        assert!(has_errors);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("digitalWrit"));
        assert!(errors[1].contains("expected ';'"));
    }

    #[test]
    fn test_parse_compilation_output_empty() {
        let output = "";

        let (has_errors, errors, warnings, success_msg) = parse_compilation_output(output);

        assert!(!has_errors);
        assert!(errors.is_empty());
        assert!(warnings.is_empty());
        assert!(success_msg.is_none());
    }

    #[test]
    fn test_parse_compilation_output_mixed() {
        let output = "sketch.ino:3:5: warning: unused variable 'x'\nsketch.ino:5:1: error: 'foo' was not declared\nsketch.ino:7:2: warning: implicit declaration of function 'bar'\ncompilation terminated.";

        let (has_errors, errors, warnings, _success_msg) = parse_compilation_output(output);

        assert!(has_errors);
        assert_eq!(errors.len(), 1);
        assert_eq!(warnings.len(), 2);
        assert!(errors[0].contains("'foo' was not declared"));
        assert!(warnings[0].contains("unused variable"));
        assert!(warnings[1].contains("implicit declaration"));
    }
}

// ============================================================================
// Command: /arduino-examples
// ============================================================================

/// Lists available example sketches from installed libraries
pub fn run_examples_command(
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput, String> {
    let worktree = worktree.ok_or("No workspace available".to_string())?;

    let mut output = String::new();
    let mut sections = Vec::new();

    output.push_str("# Arduino Example Sketches\n\n");

    // Get arduino-cli path
    let cli_path = utils::get_string_setting(worktree, "cli.path", "");
    let cli_command = if !cli_path.is_empty() {
        cli_path
    } else {
        "arduino-cli".to_string()
    };

    // First, get list of libraries to find their examples
    let lib_result = std::process::Command::new(&cli_command)
        .arg("lib")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output();

    match lib_result {
        Ok(cmd_output) => {
            if cmd_output.status.success() {
                let section_start = output.len();
                output.push_str("## Available Examples\n\n");

                let stdout = String::from_utf8_lossy(&cmd_output.stdout);

                // Try to parse JSON output
                if let Ok(libs_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(libs_array) = libs_json
                        .get("installed_libraries")
                        .and_then(|v| v.as_array())
                    {
                        if libs_array.is_empty() {
                            output.push_str("⚠️ **No libraries installed**\n\n");
                            output
                                .push_str("Install libraries to get access to example sketches.\n");
                        } else {
                            let mut total_examples = 0;
                            let mut libs_with_examples = Vec::new();

                            // Collect libraries with examples
                            for lib in libs_array {
                                if let Some(name) = lib
                                    .get("library")
                                    .and_then(|v| v.get("name"))
                                    .and_then(|v| v.as_str())
                                {
                                    if let Some(install_dir) =
                                        lib.get("install_dir").and_then(|v| v.as_str())
                                    {
                                        let examples_dir = format!("{}/examples", install_dir);

                                        if let Ok(entries) = std::fs::read_dir(&examples_dir) {
                                            let examples: Vec<String> = entries
                                                .filter_map(|e| e.ok())
                                                .filter(|e| e.path().is_dir())
                                                .filter_map(|e| {
                                                    e.file_name().to_str().map(|s| s.to_string())
                                                })
                                                .collect();

                                            if !examples.is_empty() {
                                                total_examples += examples.len();
                                                libs_with_examples
                                                    .push((name.to_string(), examples));
                                            }
                                        }
                                    }
                                }
                            }

                            if libs_with_examples.is_empty() {
                                output.push_str("⚠️ **No examples found**\n\n");
                                output.push_str(
                                    "The installed libraries don't contain example sketches.\n",
                                );
                            } else {
                                output.push_str(&format!(
                                    "✓ **Found {} example(s) from {} library/libraries**\n\n",
                                    total_examples,
                                    libs_with_examples.len()
                                ));

                                // Display examples grouped by library
                                for (lib_name, examples) in libs_with_examples {
                                    output.push_str(&format!(
                                        "### {} ({} examples)\n\n",
                                        lib_name,
                                        examples.len()
                                    ));

                                    for example in examples {
                                        output.push_str(&format!("- `{}`\n", example));
                                    }
                                    output.push_str("\n");
                                }

                                // Add usage tip
                                output.push_str("\n**To use an example:**\n");
                                output.push_str("1. Copy the example folder to your workspace\n");
                                output.push_str("2. Or reference it when creating a new sketch\n");
                                output.push_str("3. Examples are typically in: `~/Arduino/libraries/<LibName>/examples/`\n");
                            }
                        }
                    }
                } else {
                    output.push_str("⚠️ **Could not parse library list**\n\n");
                    output.push_str("Try running: `arduino-cli lib list`\n");
                }

                sections.push(zed::SlashCommandOutputSection {
                    range: zed::Range {
                        start: section_start as u32,
                        end: output.len() as u32,
                    },
                    label: "Available Examples".to_string(),
                });
            } else {
                let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                output.push_str(&format!(
                    "❌ **Failed to list libraries**\n\n```\n{}\n```\n",
                    stderr
                ));
            }
        }
        Err(e) => {
            output.push_str(&format!(
                "❌ **Failed to run arduino-cli**\n\nError: {}\n\n",
                e
            ));
            output.push_str("**Possible causes:**\n");
            output.push_str("  - Arduino CLI is not installed or not in PATH\n");
            output.push_str("  - Check `cli.path` setting if using custom location\n");
        }
    }

    Ok(zed::SlashCommandOutput {
        text: output,
        sections,
    })
}
