// Utility functions for LSP settings, argument parsing, and paths.

use zed_extension_api::{self as zed, settings::LspSettings};

// ============================================================================
// Settings
// ============================================================================

/// Navigate nested settings using dot notation and extract a boolean value
fn get_nested_bool(settings: &zed::serde_json::Value, path: &str) -> Option<bool> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = settings;

    for part in parts {
        current = current.get(part)?;
    }

    current.as_bool()
}

/// Navigate nested settings using dot notation and extract a string value
fn get_nested_string(settings: &zed::serde_json::Value, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = settings;

    for part in parts {
        current = current.get(part)?;
    }

    current.as_str().map(String::from)
}

/// Navigate nested settings using dot notation and extract an array value
fn get_nested_array(settings: &zed::serde_json::Value, path: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = settings;

    for part in parts {
        current = current.get(part)?;
    }

    let array = current.as_array()?;
    Some(
        array
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
    )
}

/// Get boolean setting from LSP config (supports nested paths like "cli.enabled")
pub fn get_setting(worktree: &zed::Worktree, key: &str, default: bool) -> bool {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|lsp| lsp.settings)
        .and_then(|s| get_nested_bool(&s, key))
        .unwrap_or(default)
}

/// Get string setting from LSP config (supports nested paths like "cli.path")
pub fn get_string_setting(worktree: &zed::Worktree, key: &str, default: &str) -> String {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|lsp| lsp.settings)
        .and_then(|s| get_nested_string(&s, key))
        .unwrap_or_else(|| default.to_string())
}

/// Get array of strings setting from LSP config (supports nested paths like "cli.arguments")
pub fn get_string_array_setting(worktree: &zed::Worktree, key: &str) -> Vec<String> {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|lsp| lsp.settings)
        .and_then(|s| get_nested_array(&s, key))
        .unwrap_or_default()
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
    get_env(worktree, "HOME").or_else(|| get_env(worktree, "USERPROFILE"))
}

/// Get environment variable value
pub fn get_env(worktree: &zed::Worktree, key: &str) -> Option<String> {
    use std::collections::HashMap;
    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    shell_env.get(key).cloned()
}
