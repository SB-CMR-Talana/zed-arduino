//! Arduino CLI wrapper functions for FQBN validation, core management, compilation, and board detection.

use std::process::Command;
use zed_extension_api::{self as zed, serde_json, Result};

// ============================================================================
// VALIDATION
// ============================================================================

/// Validate FQBN format (vendor:architecture:board or vendor:architecture:board:options)
pub fn validate_fqbn(fqbn: &str) -> Result<()> {
    let parts: Vec<&str> = fqbn.split(':').collect();
    if parts.len() < 3 {
        return Err(format!(
            "Invalid FQBN format '{}'. Expected format: 'vendor:architecture:board' (e.g., 'arduino:avr:uno')",
            fqbn
        ));
    }

    // Check that parts are not empty
    if parts[0].is_empty() || parts[1].is_empty() || parts[2].is_empty() {
        return Err(format!(
            "Invalid FQBN format '{}'. Vendor, architecture, and board cannot be empty.",
            fqbn
        ));
    }

    Ok(())
}

/// Extract core ID from FQBN (e.g., "esp32:esp32" from "esp32:esp32:esp32s3")
pub fn extract_core_id(fqbn: &str) -> Option<String> {
    let parts: Vec<&str> = fqbn.split(':').collect();
    if parts.len() >= 2 {
        Some(format!("{}:{}", parts[0], parts[1]))
    } else {
        None
    }
}

// ============================================================================
// CORE MANAGEMENT
// ============================================================================

/// Check if board core is installed via `arduino-cli core list`
pub fn is_core_installed(cli_path: &str, core_id: &str) -> bool {
    let output = Command::new(cli_path).arg("core").arg("list").output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                return stdout.contains(core_id);
            }
        }
    }
    false
}

/// Install board core (can be slow, cores are 100MB+)
pub fn install_core(cli_path: &str, core_id: &str, config_path: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(cli_path);
    cmd.arg("core").arg("install").arg(core_id);

    if let Some(config) = config_path {
        cmd.arg("--config-file").arg(config);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run arduino-cli core install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("failed to install core {}: {}", core_id, stderr));
    }

    Ok(())
}

// ============================================================================
// COMPILATION
// ============================================================================

/// Generate compile_commands.json for clangd (10-30 seconds)
pub fn generate_compilation_database(
    cli_path: &str,
    fqbn: &str,
    config_path: Option<&str>,
    library_paths: &[String],
    worktree: &zed::Worktree,
) -> Result<()> {
    let worktree_root = worktree.root_path();

    let mut cmd = Command::new(cli_path);
    cmd.arg("compile")
        .arg("--fqbn")
        .arg(fqbn)
        .arg("--only-compilation-database")
        .arg(worktree_root);

    if let Some(config) = config_path {
        cmd.arg("--config-file").arg(config);
    }

    if !library_paths.is_empty() {
        cmd.arg("--libraries").arg(library_paths.join(","));
    }

    // Add custom arduino-cli compile arguments from settings
    let custom_args = crate::utils::get_string_array_setting(worktree, "cli.compileArguments");
    for arg in custom_args {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run arduino-cli compile: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "failed to generate compilation database: {}",
            stderr
        ));
    }

    Ok(())
}

// ============================================================================
// BOARD DETECTION
// ============================================================================

/// Parse board entry from JSON and extract port address and FQBN
fn parse_board_entry(board: &serde_json::Value) -> Option<(String, String)> {
    let port_obj = board.get("port")?;
    let boards_array = board.get("matching_boards")?.as_array()?;
    let port_addr = port_obj.get("address")?.as_str()?;
    let fqbn = boards_array.first()?.get("fqbn")?.as_str()?;
    Some((port_addr.to_string(), fqbn.to_string()))
}

/// Warn user when multiple boards are detected
fn warn_multiple_boards(boards: &[(String, String)]) {
    eprintln!("Arduino: Warning - Multiple boards detected:");
    for (port, fqbn) in boards {
        eprintln!("  - {} on {}", fqbn, port);
    }
    eprintln!(
        "Arduino: Using first board: {} on {}",
        boards[0].1, boards[0].0
    );
    eprintln!("Arduino: To use a different board, edit .zed/settings.json manually");
}

/// Detect connected Arduino board and return (port, FQBN)
pub fn detect_connected_board(cli_path: &str) -> Option<(String, String)> {
    let output = Command::new(cli_path)
        .arg("board")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let boards_json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
    let boards_array = boards_json.as_array()?;

    let detected_boards: Vec<(String, String)> =
        boards_array.iter().filter_map(parse_board_entry).collect();

    let first_board = detected_boards.first()?;

    if detected_boards.len() > 1 {
        warn_multiple_boards(&detected_boards);
    }

    Some(first_board.clone())
}
