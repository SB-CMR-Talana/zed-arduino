//! Tool infrastructure: shared utilities for tool detection, download, versioning, and validation.

use std::fs;
use std::path::PathBuf;
use zed_extension_api::{self as zed};

// ============================================================================
// Shared Types
// ============================================================================

/// Information about a detected tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub path: String,
}

// ============================================================================
// File System Utilities
// ============================================================================

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// Check if a file exists and is executable (or exists on Windows)
pub fn is_executable(path: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        file_exists(path)
    }
}

/// Get absolute path from relative path
pub fn get_absolute_path(path: &str) -> Result<String, String> {
    PathBuf::from(path)
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path {}: {}", path, e))
        .and_then(|p| {
            p.to_str()
                .map(String::from)
                .ok_or_else(|| "Path is not valid UTF-8".to_string())
        })
}

// ============================================================================
// Version Utilities
// ============================================================================

/// Extract version from tool output (assumes first line contains version)
pub fn extract_version_from_output(output: &str) -> Option<String> {
    output
        .lines()
        .next()?
        .split_whitespace()
        .find(|s| s.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .map(|s| s.trim_end_matches(',').trim_end_matches(')').to_string())
}

/// Compare version strings (semantic versioning-like)
/// Returns true if version >= min_version
pub fn version_meets_minimum(version: &str, min_version: &str) -> bool {
    let parse_version =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

    let version_parts = parse_version(version);
    let min_parts = parse_version(min_version);

    for (v, m) in version_parts.iter().zip(min_parts.iter()) {
        if v > m {
            return true;
        }
        if v < m {
            return false;
        }
    }

    version_parts.len() >= min_parts.len()
}

// ============================================================================
// Tool Info Creation
// ============================================================================

/// Create ToolInfo for a detected tool
pub fn make_tool_info(path: String, _source: &str, _min_version: &str) -> ToolInfo {
    ToolInfo { path }
}

// ============================================================================
// Download Utilities
// ============================================================================

/// Check cached version status
pub enum CachedVersionStatus {
    Valid,
    NeedsUpdate,
    VersionMismatch,
}

pub fn check_cached_version<F>(
    cached_path: &Option<String>,
    requested_version: &Option<String>,
    extract_version_fn: F,
) -> CachedVersionStatus
where
    F: Fn(&str) -> Option<String>,
{
    let cached = match cached_path {
        Some(p) => p,
        None => return CachedVersionStatus::NeedsUpdate,
    };

    if !file_exists(cached) {
        return CachedVersionStatus::NeedsUpdate;
    }

    let requested = match requested_version {
        Some(v) => v,
        None => return CachedVersionStatus::Valid,
    };

    match extract_version_fn(cached) {
        Some(cached_ver) if &cached_ver == requested => CachedVersionStatus::Valid,
        Some(_) => CachedVersionStatus::VersionMismatch,
        None => CachedVersionStatus::NeedsUpdate,
    }
}

/// Cleanup old version directories (keep only current version)
pub fn cleanup_old_versions(prefix: &str, current_version_dir: &str) -> Result<(), String> {
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let entries =
        fs::read_dir(&current_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        if let Ok(name) = entry.file_name().into_string() {
            if name.starts_with(prefix) && name != current_version_dir {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }

    Ok(())
}

/// Fetch specific GitHub release version
pub fn fetch_github_version(repo: &str, version: &str) -> Result<zed::GithubRelease, String> {
    let release = zed::github_release_by_tag_name(repo, version)
        .map_err(|e| format!("Failed to fetch version {}: {}", version, e))?;

    Ok(release)
}

/// Validate binary and report result
pub fn validate_and_report_binary<F>(
    path: &str,
    tool_name: &str,
    validator: F,
    auto_download: bool,
) -> Result<(), String>
where
    F: Fn(&str) -> Result<String, String>,
{
    match validator(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            let recovery = if auto_download {
                "Extension will retry download automatically."
            } else {
                "Enable 'autoDownloadCli' in settings to auto-download."
            };
            Err(format!(
                "{} validation failed: {}. {}",
                tool_name, e, recovery
            ))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_from_output() {
        // Test arduino-cli version output
        assert_eq!(
            extract_version_from_output(
                "arduino-cli  Version: 0.35.3 Commit: 95cfd654 Date: 2024-01-18T16:22:52Z"
            ),
            Some("0.35.3".to_string())
        );

        // Test clangd version output
        assert_eq!(
            extract_version_from_output("clangd version 14.0.0"),
            Some("14.0.0".to_string())
        );

        // Test with trailing comma
        assert_eq!(
            extract_version_from_output("tool version 1.2.3,"),
            Some("1.2.3".to_string())
        );

        // Test with parentheses
        assert_eq!(
            extract_version_from_output("tool 2.0.0)"),
            Some("2.0.0".to_string())
        );

        // Test with no version
        assert_eq!(extract_version_from_output("no version here"), None);

        // Test empty string
        assert_eq!(extract_version_from_output(""), None);
    }

    #[test]
    fn test_version_meets_minimum() {
        // Equal versions
        assert!(version_meets_minimum("1.0.0", "1.0.0"));

        // Higher major version
        assert!(version_meets_minimum("2.0.0", "1.0.0"));
        assert!(!version_meets_minimum("1.0.0", "2.0.0"));

        // Higher minor version
        assert!(version_meets_minimum("1.2.0", "1.1.0"));
        assert!(!version_meets_minimum("1.1.0", "1.2.0"));

        // Higher patch version
        assert!(version_meets_minimum("1.0.2", "1.0.1"));
        assert!(!version_meets_minimum("1.0.1", "1.0.2"));

        // Different number of parts
        assert!(version_meets_minimum("1.0.0.1", "1.0.0"));
        assert!(version_meets_minimum("1.0.1", "1.0"));

        // Real-world examples
        assert!(version_meets_minimum("14.0.0", "14.0.0")); // clangd minimum
        assert!(version_meets_minimum("15.0.0", "14.0.0"));
        assert!(!version_meets_minimum("13.0.0", "14.0.0"));

        assert!(version_meets_minimum("0.35.3", "0.33.0")); // arduino-cli
        assert!(version_meets_minimum("1.0.0", "0.33.0"));
        assert!(!version_meets_minimum("0.32.0", "0.33.0"));
    }

    #[test]
    fn test_file_exists() {
        // Test with a path that should exist
        assert!(file_exists("."));
        assert!(file_exists("./src"));

        // Test with a path that shouldn't exist
        assert!(!file_exists("/this/path/should/not/exist"));
    }
}
