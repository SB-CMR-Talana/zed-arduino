use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result};

enum CachedVersionStatus {
    Valid,
    NeedsUpdate,
    VersionMismatch,
}

fn check_cached_version(
    cached_path: &Option<String>,
    desired_version: &Option<String>,
    version_extractor: fn(&str) -> Option<String>,
    tool_name: &str,
) -> CachedVersionStatus {
    if let Some(path) = cached_path.as_ref() {
        if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
            if let Some(cached_version) = version_extractor(path) {
                match desired_version {
                    Some(desired) if cached_version == *desired => {
                        return CachedVersionStatus::Valid;
                    }
                    None => {
                        return CachedVersionStatus::NeedsUpdate;
                    }
                    Some(desired) => {
                        eprintln!(
                            "Arduino: Cached {} version ({}) doesn't match desired version ({}), re-downloading...",
                            tool_name, cached_version, desired
                        );
                        return CachedVersionStatus::VersionMismatch;
                    }
                }
            }
        }
    }
    CachedVersionStatus::NeedsUpdate
}

fn get_absolute_path(relative_path: &str) -> Result<String> {
    let work_dir =
        std::env::current_dir().map_err(|e| format!("failed to get work directory: {e}"))?;
    Ok(work_dir.join(relative_path).to_string_lossy().to_string())
}

fn validate_and_report_binary(
    path: &str,
    tool_name: &str,
    validator: fn(&str) -> Result<String, String>,
    auto_download: bool,
) -> Result<()> {
    match validator(path) {
        Ok(version_info) => {
            eprintln!("Arduino: {} validated: {}", tool_name, version_info);
            Ok(())
        }
        Err(e) => {
            let error_msg =
                crate::validation::format_dependency_error(tool_name, &e, auto_download);
            eprintln!("{}", error_msg);
            Err(format!("{} validation failed: {}", tool_name, e))
        }
    }
}

fn platform_strings(platform: zed::Os, arch: zed::Architecture) -> (&'static str, &'static str) {
    let os_str = match platform {
        zed::Os::Mac => "macOS",
        zed::Os::Linux => "Linux",
        zed::Os::Windows => "Windows",
    };
    let arch_str = match arch {
        zed::Architecture::Aarch64 => "ARM64",
        zed::Architecture::X86 => "32bit",
        zed::Architecture::X8664 => "64bit",
    };
    (os_str, arch_str)
}

/// Get Arduino Language Server binary (checks PATH, downloads from GitHub if needed)
pub fn get_language_server_binary(
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which("arduino_language_server") {
        return Ok(path);
    }

    // Get custom GitHub repo from settings (format: "owner/repo"), default to official repo
    let repo =
        crate::utils::get_string_setting(worktree, "githubRepo", "arduino/arduino-language-server");

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "languageServerVersion", "");
    let version_to_use = if pinned_version.is_empty() {
        None
    } else {
        Some(pinned_version)
    };

    if let CachedVersionStatus::Valid = check_cached_version(
        cached_path,
        &version_to_use,
        crate::utils::extract_language_server_version,
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
        // Download specific version
        fetch_specific_version(&repo, version).map_err(|e| {
            let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
            crate::validation::format_dependency_error(
                "arduino-language-server",
                &format!("Failed to fetch version {}: {}", version, e),
                auto_download,
            )
        })?
    } else {
        // Download latest version
        zed::latest_github_release(
            &repo,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ).map_err(|e| {
            let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
            crate::validation::format_dependency_error(
                "arduino-language-server",
                &format!("Failed to fetch latest release: {}. This may be due to GitHub API rate limits.", e),
                auto_download
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
    let final_binary_path = format!("{}/{}", version_dir, binary_name);

    if !fs::metadata(&final_binary_path).is_ok_and(|stat| stat.is_file()) {
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

        cleanup_old_versions("arduino-language-server-", &version_dir)?;

        zed::make_file_executable(&final_binary_path)?;
        eprintln!(
            "Arduino: Language Server v{} installed successfully",
            release.version
        );
    }

    let absolute_path = get_absolute_path(&final_binary_path)?;

    let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
    validate_and_report_binary(
        &absolute_path,
        "arduino-language-server",
        crate::validation::validate_language_server,
        auto_download,
    )?;

    *cached_path = Some(absolute_path.clone());
    Ok(absolute_path)
}

/// Get arduino-cli binary (checks PATH, downloads from GitHub if needed)
pub fn get_arduino_cli_binary(
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which("arduino-cli") {
        return Ok(path);
    }

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "arduinoCliVersion", "");
    let version_to_use = if pinned_version.is_empty() {
        None
    } else {
        Some(pinned_version)
    };

    if let CachedVersionStatus::Valid = check_cached_version(
        cached_path,
        &version_to_use,
        crate::utils::extract_arduino_cli_version,
        "arduino-cli",
    ) {
        return Ok(cached_path.as_ref().unwrap().clone());
    }

    if let Some(version) = &version_to_use {
        eprintln!(
            "Arduino: Using pinned version {} for arduino-cli...",
            version
        );
    } else {
        eprintln!("Arduino: arduino-cli not found in PATH, downloading...");
    }

    let release = if let Some(ref version) = version_to_use {
        // Download specific version
        fetch_specific_version_cli(version).map_err(|e| {
            crate::validation::format_dependency_error(
                "arduino-cli",
                &format!("Failed to fetch version {}: {}", version, e),
                true,
            )
        })?
    } else {
        // Download latest version
        zed::latest_github_release(
            "arduino/arduino-cli",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            crate::validation::format_dependency_error(
                "arduino-cli",
                &format!("Failed to fetch latest release: {}", e),
                true,
            )
        })?
    };

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

    if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
        let file_type = match platform {
            zed::Os::Windows => zed::DownloadedFileType::Zip,
            _ => zed::DownloadedFileType::GzipTar,
        };

        eprintln!("Arduino: Downloading arduino-cli v{}...", version);
        zed::download_file(&asset.download_url, &version_dir, file_type).map_err(|e| {
            crate::validation::format_dependency_error(
                "arduino-cli",
                &format!("Download failed: {}", e),
                true,
            )
        })?;

        zed::make_file_executable(&binary_path)?;

        cleanup_old_versions("arduino-cli-", &version_dir)?;
        eprintln!("Arduino: arduino-cli v{} installed successfully", version);
    }

    let absolute_path = get_absolute_path(&binary_path)?;

    validate_and_report_binary(
        &absolute_path,
        "arduino-cli",
        crate::validation::validate_arduino_cli,
        true,
    )?;

    *cached_path = Some(absolute_path.clone());
    Ok(absolute_path)
}

/// Get clangd binary (checks PATH, Zed-managed locations, downloads from GitHub if needed)
pub fn get_clangd_binary(
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    // First check PATH
    if let Some(path) = worktree.which("clangd") {
        return Ok(path);
    }

    // Check Zed-managed locations
    if let Some(path) = crate::detection::find_clangd(worktree) {
        return Ok(path);
    }

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "clangdVersion", "");
    let version_to_use = if pinned_version.is_empty() {
        None
    } else {
        Some(pinned_version)
    };

    if let CachedVersionStatus::Valid = check_cached_version(
        cached_path,
        &version_to_use,
        crate::utils::extract_clangd_version,
        "clangd",
    ) {
        return Ok(cached_path.as_ref().unwrap().clone());
    }

    if let Some(version) = &version_to_use {
        eprintln!("Arduino: Using pinned version {} for clangd...", version);
    } else {
        eprintln!("Arduino: clangd not found, downloading...");
    }

    let release = if let Some(ref version) = version_to_use {
        // Download specific version
        fetch_specific_version_clangd(version).map_err(|e| {
            crate::validation::format_dependency_error(
                "clangd",
                &format!("Failed to fetch version {}: {}", version, e),
                false,
            )
        })?
    } else {
        // Download latest version
        zed::latest_github_release(
            "clangd/clangd",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            crate::validation::format_dependency_error(
                "clangd",
                &format!("Failed to fetch latest release: {}", e),
                false,
            )
        })?
    };

    let (platform, arch) = zed::current_platform();

    // clangd uses different naming conventions
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
        .map_err(|e| {
            crate::validation::format_dependency_error(
                "clangd",
                &format!("Download failed: {}", e),
                false,
            )
        })?;

        zed::make_file_executable(&binary_path)?;

        cleanup_old_versions("clangd-", &version_dir)?;
        eprintln!("Arduino: clangd v{} installed successfully", version);
    }

    let absolute_path = get_absolute_path(&binary_path)?;

    validate_and_report_binary(
        &absolute_path,
        "clangd",
        crate::validation::validate_clangd,
        false,
    )?;

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

fn fetch_github_release_by_version(
    repo: &str,
    version: &str,
    tag_prefix: &str,
) -> Result<zed::GithubRelease> {
    let tag = if version.starts_with(tag_prefix) {
        version.to_string()
    } else {
        format!("{}{}", tag_prefix, version)
    };

    zed::github_release_by_tag_name(repo, &tag).map_err(|e| {
        format!(
            "Failed to fetch version {} from {}: {}. Available versions can be found at https://github.com/{}/releases",
            version, repo, e, repo
        )
    })
}

fn fetch_specific_version(repo: &str, version: &str) -> Result<zed::GithubRelease> {
    fetch_github_release_by_version(repo, version, "v")
}

fn fetch_specific_version_cli(version: &str) -> Result<zed::GithubRelease> {
    fetch_github_release_by_version("arduino/arduino-cli", version, "v")
}

fn fetch_specific_version_clangd(version: &str) -> Result<zed::GithubRelease> {
    fetch_github_release_by_version("clangd/clangd", version, "release_")
}
