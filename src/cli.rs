use std::process::Command;
use zed_extension_api::{self as zed, serde_json, Result};

/// Extract core ID from FQBN (e.g., "esp32:esp32" from "esp32:esp32:esp32s3")
pub fn extract_core_id(fqbn: &str) -> Option<String> {
    let parts: Vec<&str> = fqbn.split(':').collect();
    if parts.len() >= 2 {
        Some(format!("{}:{}", parts[0], parts[1]))
    } else {
        None
    }
}

/// Validate FQBN format (should be vendor:architecture:board or vendor:architecture:board:options)
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

/// Generate compile_commands.json for clangd (10-30 seconds)
pub fn generate_compilation_database(
    cli_path: &str,
    fqbn: &str,
    config_path: Option<&str>,
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

/// Detect connected Arduino board and return (port, fqbn) if found
/// If multiple boards are detected, returns the first one and logs a warning
pub fn detect_connected_board(cli_path: &str) -> Option<(String, String)> {
    let output = Command::new(cli_path)
        .arg("board")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Parse JSON output to extract port and FQBN
                // JSON format: [{"matching_boards":[{"fqbn":"..."}],"port":{"address":"..."}}]
                if let Ok(boards) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(array) = boards.as_array() {
                        // Collect all detected boards
                        let mut detected_boards: Vec<(String, String)> = Vec::new();

                        for board in array {
                            if let (Some(port_obj), Some(boards_array)) = (
                                board.get("port"),
                                board.get("matching_boards").and_then(|b| b.as_array()),
                            ) {
                                if let (Some(port_addr), Some(fqbn)) = (
                                    port_obj.get("address").and_then(|a| a.as_str()),
                                    boards_array
                                        .first()
                                        .and_then(|b| b.get("fqbn"))
                                        .and_then(|f| f.as_str()),
                                ) {
                                    detected_boards.push((port_addr.to_string(), fqbn.to_string()));
                                }
                            }
                        }

                        // Return first board, with warning if multiple found
                        if let Some(first_board) = detected_boards.first() {
                            if detected_boards.len() > 1 {
                                eprintln!("Arduino: Warning - Multiple boards detected:");
                                for (port, fqbn) in &detected_boards {
                                    eprintln!("  - {} on {}", fqbn, port);
                                }
                                eprintln!(
                                    "Arduino: Using first board: {} on {}",
                                    first_board.1, first_board.0
                                );
                                eprintln!("Arduino: To use a different board, edit .zed/settings.json manually");
                            }
                            return Some(first_board.clone());
                        }
                    }
                }
            }
        }
    }
    None
}
