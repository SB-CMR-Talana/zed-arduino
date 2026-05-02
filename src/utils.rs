//! Extension utilities: Zed API helpers for settings access (with nested dot notation), argument parsing, and environment variables.

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

/// Navigate nested settings using dot notation and extract an array of strings
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

/// Get boolean setting from LSP config (supports nested paths like "autoCreateConfig")
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

/// Get array of strings setting from LSP config (supports nested paths like "cli.compileArguments")
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::serde_json::json;

    #[test]
    fn test_get_nested_bool() {
        let settings = json!({
            "lsp": {
                "arduino": {
                    "autoCreateConfig": true,
                    "autoDownloadCli": false
                }
            }
        });

        // Simple nested access
        assert_eq!(
            get_nested_bool(&settings["lsp"]["arduino"], "autoCreateConfig"),
            Some(true)
        );
        assert_eq!(
            get_nested_bool(&settings["lsp"]["arduino"], "autoDownloadCli"),
            Some(false)
        );

        // Non-existent key
        assert_eq!(
            get_nested_bool(&settings["lsp"]["arduino"], "nonExistent"),
            None
        );

        // Deep nesting with dot notation - test the function directly on a flat structure
        let flat_settings = json!({
            "settings": {
                "autoCreateConfig": true
            }
        });
        assert_eq!(
            get_nested_bool(&flat_settings, "settings.autoCreateConfig"),
            Some(true)
        );
    }

    #[test]
    fn test_get_nested_string() {
        let settings = json!({
            "cli": {
                "path": "/usr/bin/arduino-cli",
                "config": "~/.arduino15/arduino-cli.yaml"
            },
            "fqbn": "arduino:avr:uno"
        });

        // Deep nested access
        assert_eq!(
            get_nested_string(&settings, "cli.path"),
            Some("/usr/bin/arduino-cli".to_string())
        );
        assert_eq!(
            get_nested_string(&settings, "cli.config"),
            Some("~/.arduino15/arduino-cli.yaml".to_string())
        );

        // Top-level access
        assert_eq!(
            get_nested_string(&settings, "fqbn"),
            Some("arduino:avr:uno".to_string())
        );

        // Non-existent path
        assert_eq!(get_nested_string(&settings, "cli.nonExistent"), None);
        assert_eq!(get_nested_string(&settings, "invalid.path.here"), None);
    }

    #[test]
    fn test_get_nested_array() {
        let settings = json!({
            "cli": {
                "compileArguments": ["--warnings", "all", "-v"],
                "uploadArguments": ["--verbose"]
            },
            "settings": {
                "libraryPaths": ["/path/one", "/path/two", "/path/three"]
            }
        });

        // Array access
        assert_eq!(
            get_nested_array(&settings, "cli.compileArguments"),
            Some(vec![
                "--warnings".to_string(),
                "all".to_string(),
                "-v".to_string()
            ])
        );
        assert_eq!(
            get_nested_array(&settings, "cli.uploadArguments"),
            Some(vec!["--verbose".to_string()])
        );
        assert_eq!(
            get_nested_array(&settings, "settings.libraryPaths"),
            Some(vec![
                "/path/one".to_string(),
                "/path/two".to_string(),
                "/path/three".to_string()
            ])
        );

        // Empty array
        let empty_settings = json!({ "empty": [] });
        assert_eq!(get_nested_array(&empty_settings, "empty"), Some(vec![]));

        // Non-existent path
        assert_eq!(get_nested_array(&settings, "nonExistent"), None);

        // Non-array value
        let wrong_type = json!({ "notArray": "string" });
        assert_eq!(get_nested_array(&wrong_type, "notArray"), None);
    }

    #[test]
    fn test_has_arg() {
        let args = vec![
            "-fqbn".to_string(),
            "arduino:avr:uno".to_string(),
            "--verbose".to_string(),
        ];

        assert!(has_arg(&args, "-fqbn"));
        assert!(has_arg(&args, "--verbose"));
        assert!(!has_arg(&args, "--nonexistent"));
        // has_arg checks for exact string match, so values are also matched
        assert!(has_arg(&args, "arduino:avr:uno"));
    }

    #[test]
    fn test_get_arg_value() {
        let args = vec![
            "-fqbn".to_string(),
            "arduino:avr:uno".to_string(),
            "-cli".to_string(),
            "/usr/bin/arduino-cli".to_string(),
            "--verbose".to_string(),
        ];

        assert_eq!(get_arg_value(&args, "-fqbn"), Some("arduino:avr:uno"));
        assert_eq!(get_arg_value(&args, "-cli"), Some("/usr/bin/arduino-cli"));
        assert_eq!(get_arg_value(&args, "--verbose"), None); // No value after flag
        assert_eq!(get_arg_value(&args, "--nonexistent"), None);
    }
}
