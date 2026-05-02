//! Arduino Language Server: detection, download, and validation.

use std::fs;
use std::process::Command;
use zed_extension_api::{self as zed, LanguageServerId, Result};

use crate::tools::{self, CachedVersionStatus, ToolInfo};

// ============================================================================
// Constants
// ============================================================================

const BINARY_NAME: &str = "arduino-language-server";
const DEFAULT_REPO: &str = "arduino/arduino-language-server";

// ============================================================================
// Public API: Detection
// ============================================================================

/// Find arduino-language-server on system, return info (path, version, source)
#[allow(dead_code)]
pub fn find(worktree: &zed::Worktree) -> Option<ToolInfo> {
    // Check PATH
    if let Some(path) = worktree.which("arduino_language_server") {
        // No minimum version for language server, always accepted
        return Some(tools::make_tool_info(path, "PATH", "0.0.0"));
    }

    None
}

// ============================================================================
// Public API: Download
// ============================================================================

/// Get arduino-language-server binary - tries system detection first, then downloads
pub fn get_or_download(
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which("arduino_language_server") {
        return Ok(path);
    }

    // Get custom GitHub repo from settings (format: "owner/repo")
    let repo = get_repo_setting(worktree);

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "ls.version", "");
    let version_to_use = if pinned_version.is_empty() {
        None
    } else {
        Some(pinned_version)
    };

    if let CachedVersionStatus::Valid = tools::check_cached_version(
        cached_path,
        &version_to_use,
        extract_version,
        "Arduino Language Server",
    ) {
        return Ok(cached_path.as_ref().unwrap().clone());
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );

    if let Some(version) = &version_to_use {
        eprintln!(
            "Arduino: Using pinned version {} for Arduino Language Server...",
            version
        );
    } else {
        eprintln!("Arduino: Checking for Arduino Language Server updates...");
    }

    let release = if let Some(ref version) = version_to_use {
        tools::fetch_github_version(&repo, version).map_err(|e| {
            let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
            format_dependency_error(
                &format!("Failed to fetch version {}: {}", version, e),
                auto_download,
            )
        })?
    } else {
        zed::latest_github_release(
            &repo,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
            format_dependency_error(
                &format!(
                    "Failed to fetch latest release: {}. This may be due to GitHub API rate limits.",
                    e
                ),
                auto_download,
            )
        })?
    };

    let (platform, arch) = zed::current_platform();

    let asset_name = format!(
        "arduino-language-server_{}_{}_{}.tar.gz",
        release.version,
        match platform {
            zed::Os::Mac => "macOS",
            zed::Os::Linux => "Linux",
            zed::Os::Windows => "Windows",
        },
        match arch {
            zed::Architecture::Aarch64 => "ARM64",
            zed::Architecture::X86 => "32bit",
            zed::Architecture::X8664 => "64bit",
        },
    );

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

    let version_dir = format!("arduino-language-server-{}", release.version);
    let binary_name = match platform {
        zed::Os::Mac | zed::Os::Linux => "arduino-language-server",
        zed::Os::Windows => "arduino-language-server.exe",
    };
    let binary_path = format!("{}/{}", version_dir, binary_name);

    if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        eprintln!(
            "Arduino: Downloading Arduino Language Server v{}...",
            release.version
        );

        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::GzipTar,
        )
        .map_err(|e| format!("failed to download file: {e}"))?;

        tools::cleanup_old_versions("arduino-language-server-", &version_dir)?;

        zed::make_file_executable(&binary_path)?;
        eprintln!(
            "Arduino: Language Server v{} installed successfully",
            release.version
        );
    }

    let absolute_path = tools::get_absolute_path(&binary_path)?;

    let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
    tools::validate_and_report_binary(&absolute_path, BINARY_NAME, validate, auto_download)?;

    *cached_path = Some(absolute_path.clone());
    Ok(absolute_path)
}

// ============================================================================
// Public API: Validation
// ============================================================================

/// Validate arduino-language-server binary works correctly
pub fn validate(path: &str) -> Result<String, String> {
    if !tools::file_exists(path) {
        return Err(format!("Binary not found at: {}", path));
    }

    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run arduino-language-server: {}", e))?;

    if !output.status.success() {
        return Err("arduino-language-server --version command failed".to_string());
    }

    let _stdout = String::from_utf8(output.stdout)
        .map_err(|_| "Invalid UTF-8 in arduino-language-server output".to_string())?;

    if let Some(version) = extract_version(path) {
        Ok(format!("version {}", version))
    } else {
        Ok("version unknown".to_string())
    }
}

/// Extract version from arduino-language-server binary
pub fn extract_version(path: &str) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    tools::extract_version_from_output(&stdout)
}

// ============================================================================
// Public API: Configuration
// ============================================================================

/// Get GitHub repo setting for language server (allows custom forks)
pub fn get_repo_setting(worktree: &zed::Worktree) -> String {
    crate::utils::get_string_setting(worktree, "ls.githubRepo", DEFAULT_REPO)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_dependency_error(error_msg: &str, auto_download: bool) -> String {
    let recovery_steps = if auto_download {
        vec![
            "1. Check your internet connection and restart Zed",
            "2. Check GitHub API rate limits: https://api.github.com/rate_limit",
            "3. Try using a custom fork in settings:",
            "   \"ls\": { \"githubRepo\": \"arduino/arduino-language-server\" }",
        ]
    } else {
        vec![
            "1. Enable 'autoDownloadCli' in extension settings, OR",
            "2. Manually download arduino-language-server:",
            "   https://github.com/arduino/arduino-language-server/releases",
            "3. Add to PATH or set 'ls.path' in settings",
        ]
    };

    format!(
        "Arduino Language Server Error: {}\n\nRecovery options:\n{}",
        error_msg,
        recovery_steps.join("\n")
    )
}
