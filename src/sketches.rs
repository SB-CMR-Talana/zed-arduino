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

    // Recursively scan for sketch directories
    if let Err(e) = scan_for_sketches(&root, &root, &mut sketches) {
        eprintln!("Arduino: Error scanning for sketches: {}", e);
    }

    // Sort by depth (count slashes), then alphabetically
    sketches.sort_by(|a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();
        depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
    });

    sketches
}

/// Recursively scan directory for Arduino sketch files
fn scan_for_sketches(
    dir: &str,
    root: &str,
    sketches: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let entries = fs::read_dir(dir)?;
    let mut has_sketch_file = false;
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "ino" || ext == "pde" {
                    has_sketch_file = true;
                }
            }
        } else if path.is_dir() {
            // Skip hidden directories and common build/dependency folders
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !name.starts_with('.')
                    && name != "node_modules"
                    && name != "build"
                    && name != "target"
                {
                    subdirs.push(path);
                }
            }
        }
    }

    // If this directory contains sketch files, add it
    if has_sketch_file {
        // Store as relative path from root
        if dir == root {
            sketches.push(".".to_string());
        } else if let Some(relative) = dir.strip_prefix(root) {
            sketches.push(relative.trim_start_matches('/').to_string());
        }
    }

    // Recurse into subdirectories
    for subdir in subdirs {
        if let Some(path_str) = subdir.to_str() {
            let _ = scan_for_sketches(path_str, root, sketches);
        }
    }

    Ok(())
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
