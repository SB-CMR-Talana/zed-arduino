use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result};

use crate::utils::platform_strings;

/// Get Arduino Language Server binary (checks PATH, downloads from GitHub if needed)
pub fn get_language_server_binary(
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which("arduino_language_server") {
        return Ok(path);
    }

    if let Some(path) = cached_path.as_ref() {
        if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
            return Ok(path.clone());
        }
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );

    // Get custom GitHub repo from settings (format: "owner/repo"), default to official repo
    let repo =
        crate::utils::get_string_setting(worktree, "githubRepo", "arduino/arduino-language-server");

    let release = zed::latest_github_release(
        &repo,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

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
    let final_binary_path = format!("{}/{}", version_dir, binary_name);

    if !fs::metadata(&final_binary_path).map_or(false, |stat| stat.is_file()) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::GzipTar,
        )
        .map_err(|e| format!("failed to download file: {e}"))?;

        cleanup_old_versions("arduino-language-server-", &version_dir)?;

        zed::make_file_executable(&final_binary_path)?;
    }

    *cached_path = Some(final_binary_path.clone());
    Ok(final_binary_path)
}

/// Get arduino-cli binary (checks PATH, downloads from GitHub if needed)
pub fn get_arduino_cli_binary(
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which("arduino-cli") {
        return Ok(path);
    }

    if let Some(path) = cached_path.as_ref() {
        if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
            return Ok(path.clone());
        }
    }

    let release = zed::latest_github_release(
        "arduino/arduino-cli",
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let (platform, arch) = zed::current_platform();
    let (os_str, arch_str) = platform_strings(platform, arch);

    let ext = match platform {
        zed::Os::Windows => "zip",
        _ => "tar.gz",
    };

    let version = release
        .version
        .strip_prefix('v')
        .unwrap_or(&release.version);

    let asset_name = format!("arduino-cli_{version}_{os_str}_{arch_str}.{ext}");

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("no release asset found matching {asset_name}"))?;

    let version_dir = format!("arduino-cli-{version}");
    let binary_name = match platform {
        zed::Os::Windows => "arduino-cli.exe",
        _ => "arduino-cli",
    };
    let binary_path = format!("{version_dir}/{binary_name}");

    if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
        let file_type = match platform {
            zed::Os::Windows => zed::DownloadedFileType::Zip,
            _ => zed::DownloadedFileType::GzipTar,
        };

        zed::download_file(&asset.download_url, &version_dir, file_type)
            .map_err(|e| format!("failed to download arduino-cli: {e}"))?;

        zed::make_file_executable(&binary_path)?;

        cleanup_old_versions("arduino-cli-", &version_dir)?;
    }

    let work_dir =
        std::env::current_dir().map_err(|e| format!("failed to get work directory: {e}"))?;
    let absolute_path = work_dir.join(&binary_path).to_string_lossy().to_string();

    *cached_path = Some(absolute_path.clone());
    Ok(absolute_path)
}

/// Clean up old versions, keeping only the current version directory
fn cleanup_old_versions(prefix: &str, current_dir: &str) -> Result<()> {
    let entries =
        fs::read_dir(".").map_err(|e| format!("failed to list working directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to load directory entry: {e}"))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("failed to get file type for {:?}: {}", entry.path(), e))?;

        if file_type.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(prefix) && name != current_dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }
    }

    Ok(())
}
