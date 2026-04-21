use std::{collections::HashMap, fs};
use zed_extension_api::{self as zed};

/// Find clangd in PATH or Zed-managed locations (Flatpak, standard, macOS)
pub fn find_clangd(worktree: &zed::Worktree) -> Option<String> {
    if let Some(path) = worktree.which("clangd") {
        return Some(path);
    }

    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    if let Some(home) = shell_env.get("HOME") {
        // Flatpak location
        let flatpak_base = format!("{}/.var/app/dev.zed.Zed/data/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&flatpak_base) {
            return Some(path);
        }

        // Standard Zed data location (non-Flatpak)
        let standard_base = format!("{}/.local/share/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&standard_base) {
            return Some(path);
        }

        // macOS Zed location
        let macos_base = format!("{}/Library/Application Support/Zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&macos_base) {
            return Some(path);
        }
    }

    let common_paths = vec![
        "/usr/bin/clangd",
        "/usr/local/bin/clangd",
        "/opt/homebrew/bin/clangd",
    ];

    for path in common_paths {
        if fs::metadata(path).is_ok() {
            return Some(path.to_string());
        }
    }

    None
}

/// Search for clangd in versioned Zed directories (e.g., clangd_22.1.0/bin/clangd)
fn search_clangd_in_directory(base_path: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let clangd_path = entry.path().join("bin/clangd");
            if clangd_path.exists() {
                if let Some(path_str) = clangd_path.to_str() {
                    return Some(path_str.to_string());
                }
            }
        }
    }
    None
}

/// Find arduino-cli config in project root or user home directory
pub fn find_arduino_cli_config(worktree: &zed::Worktree) -> Option<String> {
    let worktree_root = worktree.root_path();

    let project_configs = vec![
        format!("{}/.arduino-cli.yaml", worktree_root),
        format!("{}/arduino-cli.yaml", worktree_root),
        format!("{}/.arduino15/arduino-cli.yaml", worktree_root),
    ];

    for config_path in project_configs {
        if fs::metadata(&config_path).is_ok() {
            return Some(config_path);
        }
    }

    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    if let Some(home) = shell_env.get("HOME") {
        let user_configs = vec![
            format!("{}/.arduino15/arduino-cli.yaml", home),
            format!("{}/.arduino-cli.yaml", home),
            format!("{}/Arduino/arduino-cli.yaml", home),
        ];

        for config_path in user_configs {
            if fs::metadata(&config_path).is_ok() {
                return Some(config_path);
            }
        }
    }

    None
}

/// Check if compile_commands.json exists in project root
pub fn check_compilation_database(worktree: &zed::Worktree) -> bool {
    let worktree_root = worktree.root_path();
    let compile_db_path = format!("{}/compile_commands.json", worktree_root);
    fs::metadata(&compile_db_path).is_ok()
}
