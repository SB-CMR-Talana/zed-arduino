//! Arduino sketch detection and compilation database utilities.

use std::fs;
use zed_extension_api::{self as zed};

// ============================================================================
// Sketch Detection
// ============================================================================

/// Find all directories containing Arduino sketch files (.ino or .pde)
/// Returns paths sorted by depth (shallowest first) then alphabetically
pub fn find_directories(worktree: &zed::Worktree) -> Vec<String> {
    let root = worktree.root_path();
    let mut sketches = Vec::new();

    // Use shell command to find .ino and .pde files since fs::read_dir doesn't work in WASM
    // Find all .ino and .pde files and get their parent directories
    let output = std::process::Command::new("find")
        .arg(root)
        .arg("-type")
        .arg("f")
        .arg("(")
        .arg("-name")
        .arg("*.ino")
        .arg("-o")
        .arg("-name")
        .arg("*.pde")
        .arg(")")
        .arg("-exec")
        .arg("dirname")
        .arg("{}")
        .arg(";")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Convert ./path to path, keep . as is
                    let relative_path = line.strip_prefix("./").unwrap_or(line);
                    sketches.push(relative_path.to_string());
                }
            }
        }
    }

    // Sort by depth (count slashes), then alphabetically
    sketches.sort_by(|a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();
        depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
    });

    sketches
}

// ============================================================================
// Compilation Database
// ============================================================================

/// Check if compile_commands.json exists in project root
pub fn check_compilation_database(worktree: &zed::Worktree) -> bool {
    let worktree_root = worktree.root_path();
    let compile_db_path = format!("{}/compile_commands.json", worktree_root);
    fs::metadata(&compile_db_path).is_ok()
}
