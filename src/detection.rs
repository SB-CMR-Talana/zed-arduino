//! Detects clangd binary, arduino-cli, arduino-cli config, and compilation database locations.
//! Comprehensive detection checks environment variables, PATH, common install locations,
//! package manager paths, and tool-specific locations.
//!
//! Detection respects standard Arduino environment variables:
//! - ARDUINO_DIRECTORIES_DATA: Arduino data directory
//! - ARDUINO_DIRECTORIES_USER: Arduino user directory
//! - ARDUINO_DIRECTORIES_DOWNLOADS: Arduino downloads directory

use std::fs;
use std::path::PathBuf;
use zed_extension_api::{self as zed};

// Minimum supported versions
const MIN_CLANGD_VERSION: &str = "14.0.0";
const MIN_ARDUINO_CLI_VERSION: &str = "0.33.0";

/// Information about a detected tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub path: String,
    pub resolved_path: Option<String>, // If it's a symlink
    pub source: String,
    pub version: Option<String>,
    pub version_ok: bool,
}

// ============================================================================
// Public Detection Functions
// ============================================================================

/// Find clangd in environment, PATH, Zed-managed locations, and common system paths
/// Returns ToolInfo with path, source, and version information
pub fn find_clangd_info(worktree: &zed::Worktree) -> Option<ToolInfo> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "CLANGD_PATH") {
        if is_executable(&path) {
            return Some(make_tool_info(
                path,
                "environment variable CLANGD_PATH",
                MIN_CLANGD_VERSION,
            ));
        }
    }

    // 2. Check PATH
    if let Some(path) = worktree.which("clangd") {
        return Some(make_tool_info(path, "PATH", MIN_CLANGD_VERSION));
    }

    // 3. Check Zed-managed locations
    if let Some(home) = crate::utils::get_home(worktree) {
        // Flatpak location
        let flatpak_base = format!("{}/.var/app/dev.zed.Zed/data/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&flatpak_base) {
            return Some(make_tool_info(
                path,
                "Zed Flatpak managed",
                MIN_CLANGD_VERSION,
            ));
        }

        // Standard Zed data location (non-Flatpak)
        let standard_base = format!("{}/.local/share/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&standard_base) {
            return Some(make_tool_info(path, "Zed managed", MIN_CLANGD_VERSION));
        }

        // macOS Zed location
        let macos_base = format!("{}/Library/Application Support/Zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&macos_base) {
            return Some(make_tool_info(
                path,
                "Zed macOS managed",
                MIN_CLANGD_VERSION,
            ));
        }
    }

    // 4. Check common system installation paths
    let common_paths = vec![
        "/usr/bin/clangd",
        "/usr/local/bin/clangd",
        "/opt/homebrew/bin/clangd",
        "/opt/homebrew/opt/llvm/bin/clangd",
        "/usr/lib/llvm/bin/clangd",
        "/snap/bin/clangd",
    ];

    for path in common_paths {
        if is_executable(path) {
            return Some(make_tool_info(
                path.to_string(),
                "system installation",
                MIN_CLANGD_VERSION,
            ));
        }
    }

    None
}

/// Legacy wrapper for backward compatibility
pub fn find_clangd(worktree: &zed::Worktree) -> Option<String> {
    find_clangd_info(worktree).map(|info| {
        log_tool_info("clangd", &info);
        info.path
    })
}

/// Find arduino-cli in environment, PATH, and common installation locations
/// Returns ToolInfo with path, source, and version information
pub fn find_arduino_cli_info(worktree: &zed::Worktree) -> Option<ToolInfo> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "ARDUINO_CLI_PATH") {
        if is_executable(&path) {
            return Some(make_tool_info(
                path,
                "environment variable ARDUINO_CLI_PATH",
                MIN_ARDUINO_CLI_VERSION,
            ));
        }
    }

    // 2. Check PATH
    if let Some(path) = worktree.which("arduino-cli") {
        return Some(make_tool_info(path, "PATH", MIN_ARDUINO_CLI_VERSION));
    }

    // 3. Check common system installation paths
    let common_paths = vec![
        "/usr/bin/arduino-cli",
        "/usr/local/bin/arduino-cli",
        "/opt/homebrew/bin/arduino-cli",
        "/snap/bin/arduino-cli",
    ];

    for path in common_paths {
        if is_executable(path) {
            return Some(make_tool_info(
                path.to_string(),
                "system installation",
                MIN_ARDUINO_CLI_VERSION,
            ));
        }
    }

    // 4. Check standard Arduino environment variable directories
    if let Some(data_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_DATA") {
        let cli_path = format!("{}/arduino-cli", data_dir);
        if is_executable(&cli_path) {
            return Some(make_tool_info(
                cli_path,
                "ARDUINO_DIRECTORIES_DATA",
                MIN_ARDUINO_CLI_VERSION,
            ));
        }
    }

    if let Some(user_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_USER") {
        let cli_path = format!("{}/arduino-cli", user_dir);
        if is_executable(&cli_path) {
            return Some(make_tool_info(
                cli_path,
                "ARDUINO_DIRECTORIES_USER",
                MIN_ARDUINO_CLI_VERSION,
            ));
        }
    }

    // 5. Check Arduino IDE installation locations
    if let Some(home) = crate::utils::get_home(worktree) {
        let ide_locations = vec![
            // Linux Arduino IDE 2.x
            format!("{}/.arduino15/arduino-cli", home),
            format!("{}/Arduino/arduino-cli", home),
            // Flatpak Arduino IDE
            format!("{}/.var/app/cc.arduino.IDE2/data/Arduino15/arduino-cli", home),
            // Snap Arduino IDE
            format!("{}/snap/arduino/current/arduino-cli", home),
            // macOS Arduino IDE
            format!("{}/Library/Arduino15/arduino-cli", home),
            format!("{}/Applications/Arduino.app/Contents/MacOS/arduino-cli", home),
            // Windows paths (will only work on Windows)
            format!("{}\\AppData\\Local\\Arduino15\\arduino-cli.exe", home),
            format!("{}\\AppData\\Local\\Programs\\Arduino IDE\\resources\\app\\lib\\backend\\resources\\arduino-cli.exe", home),
        ];

        for path in ide_locations {
            if is_executable(&path) {
                return Some(make_tool_info(
                    path,
                    "Arduino IDE installation",
                    MIN_ARDUINO_CLI_VERSION,
                ));
            }
        }
    }

    None
}

/// Legacy wrapper for backward compatibility
pub fn find_arduino_cli(worktree: &zed::Worktree) -> Option<String> {
    find_arduino_cli_info(worktree).map(|info| {
        log_tool_info("arduino-cli", &info);
        info.path
    })
}

/// Find arduino-cli config in environment, near CLI binary, project root, or user home
pub fn find_arduino_cli_config(worktree: &zed::Worktree, cli_path: Option<&str>) -> Option<String> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "ARDUINO_CLI_CONFIG") {
        if file_exists_internal(&path) {
            eprintln!("Arduino: Using config from ARDUINO_CLI_CONFIG: {}", path);
            return Some(path);
        }
    }

    // 2. Check standard Arduino directory environment variables
    if let Some(data_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_DATA") {
        let config_path = format!("{}/arduino-cli.yaml", data_dir);
        if file_exists_internal(&config_path) {
            eprintln!(
                "Arduino: Found config in ARDUINO_DIRECTORIES_DATA: {}",
                config_path
            );
            return Some(config_path);
        }
    }

    if let Some(user_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_USER") {
        let config_path = format!("{}/arduino-cli.yaml", user_dir);
        if file_exists_internal(&config_path) {
            eprintln!(
                "Arduino: Found config in ARDUINO_DIRECTORIES_USER: {}",
                config_path
            );
            return Some(config_path);
        }
    }

    // 3. Check near arduino-cli binary (if provided)
    if let Some(cli) = cli_path {
        if let Some(parent) = PathBuf::from(cli).parent() {
            let config_near_cli = parent.join("arduino-cli.yaml");
            if file_exists_internal(&config_near_cli.to_string_lossy()) {
                let path = config_near_cli.to_string_lossy().to_string();
                eprintln!("Arduino: Found config near arduino-cli binary: {}", path);
                return Some(path);
            }
        }
    }

    // 4. Check project root locations
    let worktree_root = worktree.root_path();
    let project_configs = vec![
        format!("{}/.arduino-cli.yaml", worktree_root),
        format!("{}/arduino-cli.yaml", worktree_root),
        format!("{}/.arduino15/arduino-cli.yaml", worktree_root),
    ];

    for config_path in project_configs {
        if file_exists_internal(&config_path) {
            eprintln!("Arduino: Found config in project root: {}", config_path);
            return Some(config_path);
        }
    }

    // 5. Check user home standard locations
    if let Some(home) = crate::utils::get_home(worktree) {
        let user_configs = vec![
            // Standard Arduino locations
            format!("{}/.arduino15/arduino-cli.yaml", home),
            format!("{}/.arduino-cli.yaml", home),
            format!("{}/Arduino/arduino-cli.yaml", home),
            // XDG config location (Linux)
            format!("{}/.config/arduino-cli/arduino-cli.yaml", home),
            // Flatpak Arduino IDE
            format!(
                "{}/.var/app/cc.arduino.IDE2/data/Arduino15/arduino-cli.yaml",
                home
            ),
            // macOS
            format!("{}/Library/Arduino15/arduino-cli.yaml", home),
            // Windows
            format!("{}\\AppData\\Local\\Arduino15\\arduino-cli.yaml", home),
        ];

        for config_path in user_configs {
            if file_exists_internal(&config_path) {
                eprintln!("Arduino: Found config in user home: {}", config_path);
                return Some(config_path);
            }
        }
    }

    // 6. Check XDG_CONFIG_HOME on Linux
    if let Some(xdg_config) = crate::utils::get_env(worktree, "XDG_CONFIG_HOME") {
        let xdg_config_path = format!("{}/arduino-cli/arduino-cli.yaml", xdg_config);
        if file_exists_internal(&xdg_config_path) {
            eprintln!(
                "Arduino: Found config in XDG_CONFIG_HOME: {}",
                xdg_config_path
            );
            return Some(xdg_config_path);
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

/// Find all directories containing Arduino sketch files (.ino or .pde)
/// Returns paths sorted by depth (shallowest first) then alphabetically
pub fn find_sketch_directories(worktree: &zed::Worktree) -> Vec<String> {
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

/// Check if a file exists (public wrapper)
pub fn file_exists(path: &str) -> bool {
    file_exists_internal(path)
}

/// Log tool information with warnings if needed (public wrapper)
pub fn log_tool_info_public(tool_name: &str, info: &ToolInfo) {
    log_tool_info(tool_name, info);
}

// ============================================================================
// Helpers
// ============================================================================

/// Search for clangd in versioned Zed directories (e.g., clangd_22.1.0/bin/clangd)
fn search_clangd_in_directory(base_path: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(base_path) {
        let mut versions: Vec<_> = entries.flatten().collect();
        // Sort by directory name (version) in descending order to prefer newer versions
        versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for entry in versions {
            let clangd_path = entry.path().join("bin/clangd");
            if is_executable(&clangd_path.to_string_lossy()) {
                if let Some(path_str) = clangd_path.to_str() {
                    return Some(path_str.to_string());
                }
            }
        }
    }
    None
}

/// Check if a file exists
fn file_exists_internal(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// Check if a file exists and is executable (or exists on Windows)
fn is_executable(path: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.is_file() && metadata.permissions().mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        // On Windows, just check if the file exists
        file_exists_internal(path)
    }
}

/// Resolve symlinks to get the actual file path
fn resolve_symlink(path: &str) -> Option<String> {
    if let Ok(canonical) = fs::canonicalize(path) {
        if let Some(canonical_str) = canonical.to_str() {
            if canonical_str != path {
                return Some(canonical_str.to_string());
            }
        }
    }
    None
}

/// Create ToolInfo with version and symlink detection
fn make_tool_info(path: String, source: &str, min_version: &str) -> ToolInfo {
    let resolved_path = resolve_symlink(&path);
    let version = extract_version_from_binary(&path);
    let version_ok = if let Some(ref v) = version {
        compare_versions(v, min_version)
    } else {
        false
    };

    ToolInfo {
        path,
        resolved_path,
        source: source.to_string(),
        version,
        version_ok,
    }
}

/// Extract version from a binary by running it with --version
fn extract_version_from_binary(path: &str) -> Option<String> {
    use std::process::Command;

    let output = Command::new(path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Extract version number (looks for X.Y.Z pattern)
    for line in stdout.lines() {
        if let Some(version) = extract_version_number(line) {
            return Some(version);
        }
    }
    None
}

/// Extract version number from a string (looks for X.Y.Z pattern)
fn extract_version_number(text: &str) -> Option<String> {
    // Simple regex-free version parsing
    for word in text.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
        if cleaned.is_empty() {
            continue;
        }
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() >= 3 && parts.iter().take(3).all(|p| p.parse::<u32>().is_ok()) {
            return Some(cleaned.to_string());
        }
    }
    None
}

/// Compare two version strings (returns true if actual >= minimum)
fn compare_versions(actual: &str, minimum: &str) -> bool {
    let actual_parts: Vec<u32> = actual.split('.').filter_map(|s| s.parse().ok()).collect();
    let minimum_parts: Vec<u32> = minimum.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..3 {
        let a = actual_parts.get(i).copied().unwrap_or(0);
        let m = minimum_parts.get(i).copied().unwrap_or(0);

        if a > m {
            return true;
        } else if a < m {
            return false;
        }
    }
    true // Equal versions
}

/// Log tool information with warnings if needed
fn log_tool_info(tool_name: &str, info: &ToolInfo) {
    eprint!(
        "Arduino: Found {} from {}: {}",
        tool_name, info.source, info.path
    );

    if let Some(ref resolved) = info.resolved_path {
        eprint!(" -> {}", resolved);
    }

    if let Some(ref version) = info.version {
        eprint!(" (version {})", version);
        if !info.version_ok {
            eprintln!();
            eprintln!(
                "Arduino: WARNING - {} version {} may be too old (minimum recommended: {})",
                tool_name,
                version,
                if tool_name == "clangd" {
                    MIN_CLANGD_VERSION
                } else {
                    MIN_ARDUINO_CLI_VERSION
                }
            );
        } else {
            eprintln!();
        }
    } else {
        eprintln!(" (version unknown)");
    }
}

/// Generate helpful PATH hint for a tool
pub fn suggest_path_addition(tool_name: &str, tool_path: &str) -> String {
    if let Some(parent) = PathBuf::from(tool_path).parent() {
        if let Some(dir) = parent.to_str() {
            return format!(
                "\nHint: {} is not in your PATH.\nConsider adding to PATH: export PATH=\"{}:$PATH\"",
                tool_name, dir
            );
        }
    }
    String::new()
}
