use zed_extension_api::{self as zed, settings::LspSettings};

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

pub fn has_arg(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

pub fn get_arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()))
}

/// Get home directory from environment (cross-platform: HOME on Unix, USERPROFILE on Windows)
pub fn get_home(worktree: &zed::Worktree) -> Option<String> {
    use std::collections::HashMap;
    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    shell_env
        .get("HOME")
        .or_else(|| shell_env.get("USERPROFILE"))
        .cloned()
}

/// Extract version from arduino-cli binary path (e.g., "arduino-cli-1.0.4/arduino-cli" -> "1.0.4")
pub fn extract_arduino_cli_version(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return None;
    }

    let dir_name = parts[parts.len() - 2];
    if let Some(version) = dir_name.strip_prefix("arduino-cli-") {
        return Some(version.to_string());
    }

    None
}

/// Extract version from language server binary path (e.g., "arduino-language-server-0.7.5/arduino-language-server" -> "0.7.5")
pub fn extract_language_server_version(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return None;
    }

    let dir_name = parts[parts.len() - 2];
    if let Some(version) = dir_name.strip_prefix("arduino-language-server-") {
        return Some(version.to_string());
    }

    None
}

/// Extract version from clangd binary path (e.g., "clangd-18.1.3/clangd_18.1.3/bin/clangd" -> "18.1.3")
pub fn extract_clangd_version(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.is_empty() {
        return None;
    }

    let dir_name = parts[0];
    if let Some(version) = dir_name.strip_prefix("clangd-") {
        return Some(version.to_string());
    }

    None
}
