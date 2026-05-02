//! Clangd tool: detection, download, and validation.

use std::fs;
use std::process::Command;
use zed_extension_api::{self as zed, Result};

use crate::tools::{self, CachedVersionStatus, ToolInfo};

// ============================================================================
// Constants
// ============================================================================

const MIN_VERSION: &str = "14.0.0";
const BINARY_NAME: &str = "clangd";
const DEFAULT_REPO: &str = "clangd/clangd";

// ============================================================================
// Public API: Detection
// ============================================================================

/// Find clangd on system, return info (path, version, source)
pub fn find(worktree: &zed::Worktree) -> Option<ToolInfo> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "CLANGD_PATH") {
        if tools::is_executable(&path) {
            return Some(tools::make_tool_info(
                path,
                "environment variable CLANGD_PATH",
                MIN_VERSION,
            ));
        }
    }

    // 2. Check PATH
    if let Some(path) = worktree.which(BINARY_NAME) {
        return Some(tools::make_tool_info(path, "PATH", MIN_VERSION));
    }

    // 3. Check Zed-managed locations
    if let Some(home) = crate::utils::get_home(worktree) {
        // Flatpak location
        let flatpak_base = format!("{}/.var/app/dev.zed.Zed/data/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&flatpak_base) {
            return Some(tools::make_tool_info(
                path,
                "Zed Flatpak managed",
                MIN_VERSION,
            ));
        }

        // Standard Zed data location (non-Flatpak)
        let standard_base = format!("{}/.local/share/zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&standard_base) {
            return Some(tools::make_tool_info(path, "Zed managed", MIN_VERSION));
        }

        // macOS Zed location
        let macos_base = format!("{}/Library/Application Support/Zed/languages/clangd", home);
        if let Some(path) = search_clangd_in_directory(&macos_base) {
            return Some(tools::make_tool_info(
                path,
                "Zed macOS managed",
                MIN_VERSION,
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
        if tools::is_executable(path) {
            return Some(tools::make_tool_info(
                path.to_string(),
                "system installation",
                MIN_VERSION,
            ));
        }
    }

    None
}

// ============================================================================
// Public API: Download
// ============================================================================

/// Get clangd binary - tries system detection first, then downloads
pub fn get_or_download(
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    // Check PATH first
    if let Some(path) = worktree.which(BINARY_NAME) {
        return Ok(path);
    }

    // Check Zed-managed and system locations
    if let Some(info) = find(worktree) {
        return Ok(info.path);
    }

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "clangd.version", "");
    let version_to_use = if pinned_version.is_empty() {
        None
    } else {
        Some(pinned_version)
    };

    if let CachedVersionStatus::Valid =
        tools::check_cached_version(cached_path, &version_to_use, extract_version, BINARY_NAME)
    {
        return Ok(cached_path.as_ref().unwrap().clone());
    }

    if let Some(version) = &version_to_use {
        eprintln!("Arduino: Using pinned version {} for clangd...", version);
    } else {
        eprintln!("Arduino: clangd not found, downloading...");
    }

    let release = if let Some(ref version) = version_to_use {
        tools::fetch_github_version(DEFAULT_REPO, version).map_err(|e| {
            format_dependency_error(&format!("Failed to fetch version {}: {}", version, e))
        })?
    } else {
        zed::latest_github_release(
            DEFAULT_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format_dependency_error(&format!("Failed to fetch latest release: {}", e)))?
    };

    let (platform, arch) = zed::current_platform();

    // clangd uses different platform naming conventions
    let (os_str, arch_str) = match (platform, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => ("mac", "arm64"),
        (zed::Os::Mac, zed::Architecture::X8664) => ("mac", "x86_64"),
        (zed::Os::Linux, zed::Architecture::Aarch64) => ("linux", "aarch64"),
        (zed::Os::Linux, zed::Architecture::X8664) => ("linux", "x86_64"),
        (zed::Os::Windows, zed::Architecture::X8664) => ("windows", "x86_64"),
        _ => {
            return Err(format!(
                "Unsupported platform for clangd: {:?} {:?}",
                platform, arch
            ))
        }
    };

    let asset_name = format!("clangd-{}-{}.zip", os_str, arch_str);

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("no release asset found matching {}", asset_name))?;

    let version = release
        .version
        .strip_prefix("release_")
        .unwrap_or(&release.version);

    let version_dir = format!("clangd-{}", version);
    let binary_name = match platform {
        zed::Os::Windows => "clangd.exe",
        _ => "clangd",
    };
    let binary_path = format!("{}/clangd_{}/bin/{}", version_dir, version, binary_name);

    if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
        eprintln!("Arduino: Downloading clangd v{}...", version);
        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::Zip,
        )
        .map_err(|e| format_dependency_error(&format!("Download failed: {}", e)))?;

        zed::make_file_executable(&binary_path)?;

        tools::cleanup_old_versions("clangd-", &version_dir)?;
        eprintln!("Arduino: clangd v{} installed successfully", version);
    }

    let absolute_path = tools::get_absolute_path(&binary_path)?;

    tools::validate_and_report_binary(&absolute_path, BINARY_NAME, validate, false)?;

    *cached_path = Some(absolute_path.clone());
    Ok(absolute_path)
}

// ============================================================================
// Public API: Validation
// ============================================================================

/// Validate clangd binary works correctly
pub fn validate(path: &str) -> Result<String, String> {
    if !tools::file_exists(path) {
        return Err(format!("Binary not found at: {}", path));
    }

    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run clangd: {}", e))?;

    if !output.status.success() {
        return Err("clangd --version command failed".to_string());
    }

    let _stdout = String::from_utf8(output.stdout)
        .map_err(|_| "Invalid UTF-8 in clangd output".to_string())?;

    let version = extract_version(path).ok_or("Could not extract version")?;

    if !tools::version_meets_minimum(&version, MIN_VERSION) {
        return Ok(format!(
            "version {} (warning: minimum {} recommended)",
            version, MIN_VERSION
        ));
    }

    Ok(format!("version {}", version))
}

/// Extract version from clangd binary
pub fn extract_version(path: &str) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    tools::extract_version_from_output(&stdout)
}

/// Check if version meets minimum requirements
pub fn meets_minimum_version(version: &str) -> bool {
    tools::version_meets_minimum(version, MIN_VERSION)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Search for clangd in versioned Zed directories (e.g., clangd_22.1.0/bin/clangd)
fn search_clangd_in_directory(base_path: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(base_path) {
        let mut versions: Vec<_> = entries.flatten().collect();
        // Sort by directory name (version) in descending order to prefer newer versions
        versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for entry in versions {
            let clangd_path = entry.path().join("bin/clangd");
            if tools::is_executable(&clangd_path.to_string_lossy()) {
                if let Some(path_str) = clangd_path.to_str() {
                    return Some(path_str.to_string());
                }
            }
        }
    }
    None
}

fn format_dependency_error(error_msg: &str) -> String {
    let recovery_steps = vec![
        "1. Check your internet connection and restart Zed",
        "2. Check GitHub API rate limits: https://api.github.com/rate_limit",
        "3. Install clangd manually:",
        "   macOS: brew install llvm",
        "   Linux: apt install clangd or dnf install clang-tools-extra",
        "   Or download from: https://github.com/clangd/clangd/releases",
    ];

    format!(
        "Clangd Error: {}\n\nRecovery options:\n{}",
        error_msg,
        recovery_steps.join("\n")
    )
}
