//! Manages Arduino project configuration, arduino-cli isolated setup, and validates dependencies.

use std::fs;
use zed_extension_api::{self as zed, Result};

use crate::metadata::InstallationState;

// ============================================================================
// Dependency Checks
// ============================================================================

/// Check for missing dependencies and configuration issues. Returns (errors, warnings).
pub fn check_dependencies(worktree: &zed::Worktree) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let auto_download_cli = crate::utils::get_setting(worktree, "autoDownloadCli", true);
    let auto_create_config = crate::utils::get_setting(worktree, "autoCreateConfig", true);

    // Check for arduino-cli
    if worktree.which("arduino-cli").is_none() {
        if auto_download_cli {
            warnings.push(
                "arduino-cli not found in PATH. It will be auto-downloaded on first use."
                    .to_string(),
            );
        } else {
            errors.push(
                "arduino-cli not found in PATH and auto-download is disabled.\n  \
                Install with: brew install arduino-cli (macOS) or see https://arduino.github.io/arduino-cli/installation/\n  \
                Or enable auto-download in settings: \"autoDownloadCli\": true"
                    .to_string(),
            );
        }
    }

    // Check for clangd (optional but recommended)
    if crate::clangd::find(worktree).is_none() {
        warnings.push(
            "clangd not found. IntelliSense features will be limited.\n  \
            Recommended: Open any C++ file in Zed to trigger automatic clangd installation.\n  \
            Or install manually: brew install llvm (macOS) or apt install clangd (Linux)"
                .to_string(),
        );
    }

    // Check for arduino-cli config (optional)
    if crate::arduino_cli::find_config(worktree, None).is_none() {
        if auto_create_config {
            warnings.push(
                "arduino-cli.yaml not found. A minimal config will be auto-created.".to_string(),
            );
        } else {
            warnings.push(
                "arduino-cli.yaml not found. Some features may be limited.\n  \
                Run 'arduino-cli config init' or enable 'autoCreateConfig' in settings."
                    .to_string(),
            );
        }
    }

    // Check for compilation database (optional but recommended)
    if !crate::sketches::check_compilation_database(worktree) {
        warnings.push(
            "compile_commands.json not found. IntelliSense accuracy will be limited until first compile.\n  \
            It will be auto-generated when you run a compile task, or manually via task 'Arduino: Generate Compilation Database'."
                .to_string(),
        );
    }

    // Check for FQBN configuration (critical)
    let mut fqbn_configured = false;

    // First check settings.fqbn
    let fqbn_in_settings = crate::utils::get_string_setting(worktree, "fqbn", "");
    if !fqbn_in_settings.is_empty() {
        fqbn_configured = true;
    }

    // Fall back to binary.arguments (backward compatibility)
    if !fqbn_configured {
        if let Ok(lsp_settings) =
            zed_extension_api::settings::LspSettings::for_worktree("arduino", worktree)
        {
            if let Some(binary) = lsp_settings.binary {
                if let Some(args) = binary.arguments {
                    fqbn_configured = args.iter().any(|arg| arg == "-fqbn");
                }
            }
        }
    }

    if !fqbn_configured {
        errors.push(
            "FQBN not configured. The extension cannot function without it.\n  \
            Add to .zed/settings.json:\n  \
            \"lsp\": { \"arduino\": { \"settings\": { \"fqbn\": \"arduino:avr:uno\" } } }\n  \
            Or use binary.arguments (legacy): { \"binary\": { \"arguments\": [\"-fqbn\", \"arduino:avr:uno\"] } }\n  \
            Find your board's FQBN with task 'Arduino: List Boards & Ports'"
                .to_string(),
        );
    }

    (errors, warnings)
}

/// Print dependency check results to stderr
pub fn report_dependencies(worktree: &zed::Worktree) {
    let (errors, warnings) = check_dependencies(worktree);

    if !errors.is_empty() {
        eprintln!("\n❌ Arduino Extension - Critical Issues:");
        for error in &errors {
            eprintln!("\n{}", error);
        }
    }

    if !warnings.is_empty() {
        eprintln!("\n⚠️  Arduino Extension - Recommendations:");
        for warning in &warnings {
            eprintln!("\n{}", warning);
        }
    }

    if errors.is_empty() && warnings.is_empty() {
        eprintln!("✅ Arduino Extension - All dependencies configured");
    }
}

// ============================================================================
// Public Auto-Generation Functions
// ============================================================================

// ============================================================================
// Arduino Config Management
// ============================================================================

/// Create isolated arduino-cli config file that stores all data in extension directory
pub fn create_isolated_arduino_config(state: &InstallationState) -> Result<()> {
    // Only create if arduino-cli was downloaded by the extension
    if !state.arduino_cli_installed_by_extension() {
        return Ok(());
    }

    let config_file = "arduino-cli-isolated.yaml";
    let data_dir = state
        .get_arduino_cli_data_dir()
        .unwrap_or_else(|| "arduino-data".to_string());

    // Create isolated config pointing all directories to extension work directory
    let config_content = format!(
        r#"# Arduino CLI isolated configuration (managed by Zed extension)
# All data is stored in the extension's directory for clean isolation

directories:
  data: {}
  downloads: {}/staging
  user: {}/user
  builtin_tools: {}/builtin_tools

daemon:
  port: "50051"

output:
  no_color: false

board_manager:
  additional_urls: []

library:
  enable_unsafe_install: false

logging:
  level: info
  format: text

updater:
  enable_notification: false

# Note: Cache directory cannot be configured in arduino-cli config.
# On Linux: May use ~/.cache/arduino-cli/
# On macOS: May use ~/Library/Caches/arduino-cli/
# On Windows: May use %LOCALAPPDATA%\arduino-cli\
# These cache files are typically small and temporary.
"#,
        data_dir, data_dir, data_dir, data_dir
    );

    // Create data directory if it doesn't exist
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("failed to create arduino data directory: {}", e))?;

    // Write config file
    fs::write(config_file, config_content)
        .map_err(|e| format!("failed to write isolated arduino-cli config: {}", e))?;

    eprintln!(
        "Arduino: Created isolated configuration - all cores and libraries will be stored in extension directory"
    );

    Ok(())
}
