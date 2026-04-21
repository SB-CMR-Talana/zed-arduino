// Utility functions for LSP settings, argument parsing, and paths.

use zed_extension_api::{self as zed, settings::LspSettings};

// ============================================================================
// Settings
// ============================================================================

/// Get boolean setting from LSP config
pub fn get_setting(worktree: &zed::Worktree, key: &str, default: bool) -> bool {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|s| s.settings)
        .and_then(|s| s.get(key).and_then(zed::serde_json::Value::as_bool))
        .unwrap_or(default)
}

/// Get string setting from LSP config (returns default if not found)
pub fn get_string_setting(worktree: &zed::Worktree, key: &str, default: &str) -> String {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|s| s.settings)
        .and_then(|s| s.get(key).and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| default.to_string())
}

/// Get array of library paths from LSP config (returns empty vec if not found)
pub fn get_library_paths(worktree: &zed::Worktree) -> Vec<String> {
    let lsp_settings = match LspSettings::for_worktree("arduino", worktree) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let settings = match lsp_settings.settings {
        Some(s) => s,
        None => return Vec::new(),
    };

    let library_paths = match settings.get("libraryPaths") {
        Some(v) => v,
        None => return Vec::new(),
    };

    let paths_array = match library_paths.as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    paths_array
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

// ============================================================================
// Arguments
// ============================================================================

/// Check if argument flag exists in args
pub fn has_arg(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Get value following a flag in args
pub fn get_arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()))
}

// ============================================================================
// Paths
// ============================================================================

/// Get home directory from environment (HOME on Unix, USERPROFILE on Windows)
pub fn get_home(worktree: &zed::Worktree) -> Option<String> {
    use std::collections::HashMap;
    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    shell_env
        .get("HOME")
        .or_else(|| shell_env.get("USERPROFILE"))
        .cloned()
}
