//! Arduino sketch detection and compilation database utilities.

use std::fs;
use zed_extension_api::{self as zed, process::Command};

// ============================================================================
// Sketch Detection
// ============================================================================

/// Find all directories containing Arduino sketch files (.ino or .pde)
/// Returns paths sorted by depth (shallowest first) then alphabetically
///
/// A valid Arduino sketch is a directory containing at least one .ino or .pde file
pub fn find_directories(worktree: &zed::Worktree) -> Vec<String> {
    let root = worktree.root_path();
    let mut sketches = Vec::new();

    let mut cmd = Command::new("find");
    cmd = cmd.args([
        &root, "-type", "f", "(", "-name", "*.ino", "-o", "-name", "*.pde", ")", "-print",
    ]);

    match cmd.output() {
        Ok(output) => {
            if output.status == Some(0) {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse output to extract unique sketch directories
                for line in stdout.lines() {
                    if let Some(sketch_dir) = extract_sketch_directory(line, &root) {
                        if !sketches.contains(&sketch_dir) {
                            sketches.push(sketch_dir);
                        }
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Arduino: Sketch detection failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Arduino: Failed to detect sketches: {}", e);
        }
    }

    if !sketches.is_empty() {
        eprintln!("Arduino: Found {} sketch(es)", sketches.len());
    }

    // Sort by depth (shallowest first), then alphabetically
    sketches.sort_by(|a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();
        depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
    });

    sketches
}

/// Extract the sketch directory from a file path
/// Converts absolute path to relative path from workspace root
fn extract_sketch_directory(file_path: &str, root: &str) -> Option<String> {
    // Remove the root prefix to get relative path
    let relative = file_path.strip_prefix(root)?.strip_prefix('/')?;

    // Get the directory containing the .ino/.pde file
    let dir = if let Some(last_slash) = relative.rfind('/') {
        &relative[..last_slash]
    } else {
        // File is in root directory
        "."
    };

    // Skip common non-sketch directories (grammars, node_modules, etc.)
    // Note: We allow 'test' directories as they may contain valid test sketches
    if dir.contains("/grammars/")
        || dir.contains("/examples/")
        || dir.contains("/node_modules/")
        || dir.starts_with("grammars/")
        || dir.starts_with("node_modules/")
        || dir.starts_with(".")
    {
        return None;
    }

    Some(dir.to_string())
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
