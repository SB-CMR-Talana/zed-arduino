//! Arduino CLI tool: detection, download, validation, and wrapper functions.

use std::process::Command;
use zed_extension_api::{self as zed, Result};

use crate::tools::{self, CachedVersionStatus, ToolInfo};

// ============================================================================
// Constants
// ============================================================================

const MIN_VERSION: &str = "0.33.0";
const BINARY_NAME: &str = "arduino-cli";
const DEFAULT_REPO: &str = "arduino/arduino-cli";

// ============================================================================
// Public API: Detection
// ============================================================================

/// Find arduino-cli on system, return info (path, version, source)
pub fn find(worktree: &zed::Worktree) -> Option<ToolInfo> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "ARDUINO_CLI_PATH") {
        if tools::is_executable(&path) {
            return Some(tools::make_tool_info(
                path,
                "environment variable ARDUINO_CLI_PATH",
                MIN_VERSION,
            ));
        }
    }

    // 2. Check PATH
    if let Some(path) = worktree.which(BINARY_NAME) {
        return Some(tools::make_tool_info(path, "PATH", MIN_VERSION));
    }

    // 3. Check common system installation paths
    let common_paths = vec![
        "/usr/bin/arduino-cli",
        "/usr/local/bin/arduino-cli",
        "/opt/homebrew/bin/arduino-cli",
        "/snap/bin/arduino-cli",
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

    // 4. Check standard Arduino environment variable directories
    if let Some(data_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_DATA") {
        let cli_path = format!("{}/arduino-cli", data_dir);
        if tools::is_executable(&cli_path) {
            return Some(tools::make_tool_info(
                cli_path,
                "ARDUINO_DIRECTORIES_DATA",
                MIN_VERSION,
            ));
        }
    }

    if let Some(user_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_USER") {
        let cli_path = format!("{}/arduino-cli", user_dir);
        if tools::is_executable(&cli_path) {
            return Some(tools::make_tool_info(
                cli_path,
                "ARDUINO_DIRECTORIES_USER",
                MIN_VERSION,
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
            format!(
                "{}/.var/app/cc.arduino.IDE2/data/Arduino15/arduino-cli",
                home
            ),
            // Snap Arduino IDE
            format!("{}/snap/arduino/current/arduino-cli", home),
            // macOS Arduino IDE
            format!("{}/Library/Arduino15/arduino-cli", home),
            format!(
                "{}/Applications/Arduino.app/Contents/MacOS/arduino-cli",
                home
            ),
            // Windows paths (will only work on Windows)
            format!("{}\\AppData\\Local\\Arduino15\\arduino-cli.exe", home),
            format!("{}\\AppData\\Local\\Programs\\Arduino IDE\\resources\\app\\lib\\backend\\resources\\arduino-cli.exe", home),
        ];

        for path in ide_locations {
            if tools::is_executable(&path) {
                return Some(tools::make_tool_info(
                    path,
                    "Arduino IDE installation",
                    MIN_VERSION,
                ));
            }
        }
    }

    None
}

// ============================================================================
// Public API: Download
// ============================================================================

/// Get arduino-cli binary - tries system detection first, then downloads
pub fn get_or_download(
    worktree: &zed::Worktree,
    cached_path: &mut Option<String>,
) -> Result<String> {
    if let Some(path) = worktree.which(BINARY_NAME) {
        return Ok(path);
    }

    // Check for version pinning setting
    let pinned_version = crate::utils::get_string_setting(worktree, "cli.version", "");
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

    if let Some(ref version) = version_to_use {
        eprintln!(
            "Arduino: Using pinned version {} for arduino-cli...",
            version
        );
    } else {
        eprintln!("Arduino: Checking for arduino-cli updates...");
    }

    let release = if let Some(ref version) = version_to_use {
        tools::fetch_github_version(DEFAULT_REPO, version).map_err(|e| {
            let auto_download = crate::utils::get_setting(worktree, "autoDownloadCli", true);
            format_dependency_error(
                &format!("Failed to fetch version {}: {}", version, e),
                auto_download,
            )
        })?
    } else {
        zed::latest_github_release(
            DEFAULT_REPO,
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
        "arduino-cli_{}_{}_{}.tar.gz",
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

    let version_dir = format!("arduino-cli-{}", release.version);
    let binary_name = match platform {
        zed::Os::Mac | zed::Os::Linux => "arduino-cli",
        zed::Os::Windows => "arduino-cli.exe",
    };
    let binary_path = format!("{}/{}", version_dir, binary_name);

    if !std::fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
        eprintln!("Arduino: Downloading arduino-cli v{}...", release.version);

        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::GzipTar,
        )
        .map_err(|e| format!("failed to download file: {e}"))?;

        tools::cleanup_old_versions("arduino-cli-", &version_dir)?;

        zed::make_file_executable(&binary_path)?;
        eprintln!(
            "Arduino: arduino-cli v{} installed successfully",
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

/// Validate arduino-cli binary works correctly
pub fn validate(path: &str) -> Result<String, String> {
    if !tools::file_exists(path) {
        return Err(format!("Binary not found at: {}", path));
    }

    let output = Command::new(path)
        .arg("version")
        .output()
        .map_err(|e| format!("Failed to run arduino-cli: {}", e))?;

    if !output.status.success() {
        return Err("arduino-cli version command failed".to_string());
    }

    let _stdout = String::from_utf8(output.stdout)
        .map_err(|_| "Invalid UTF-8 in arduino-cli output".to_string())?;

    let version = extract_version(path).ok_or("Could not extract version")?;

    if !tools::version_meets_minimum(&version, MIN_VERSION) {
        return Ok(format!(
            "version {} (warning: minimum {} recommended)",
            version, MIN_VERSION
        ));
    }

    Ok(format!("version {}", version))
}

/// Extract version from arduino-cli binary
pub fn extract_version(path: &str) -> Option<String> {
    let output = Command::new(path).arg("version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    tools::extract_version_from_output(&stdout)
}

/// Check if version meets minimum requirements
#[allow(dead_code)]
pub fn meets_minimum_version(version: &str) -> bool {
    tools::version_meets_minimum(version, MIN_VERSION)
}

// ============================================================================
// Public API: Board Detection
// ============================================================================

/// Detect connected Arduino boards
pub fn detect_connected_board(cli_path: &str) -> Option<(String, Option<String>, Option<String>)> {
    use zed_extension_api::serde_json;

    let output = std::process::Command::new(cli_path)
        .args(["board", "list", "--format", "json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    // Parse board list - it's an array of detected boards
    let boards = json.as_array()?;

    // Get first detected board with matching_boards
    for board in boards {
        if let Some(matching_boards) = board
            .get("matching_boards")
            .and_then(|v: &serde_json::Value| v.as_array())
        {
            if let Some(first_match) = matching_boards.first() {
                let fqbn = first_match.get("fqbn")?.as_str()?.to_string();
                let port = board
                    .get("port")
                    .and_then(|p: &serde_json::Value| p.get("address"))
                    .and_then(|a: &serde_json::Value| a.as_str())
                    .map(String::from);
                let name = first_match.get("name")?.as_str().map(String::from);

                return Some((fqbn, port, name));
            }
        }
    }

    None
}

// ============================================================================
// Public API: Configuration
// ============================================================================

/// Find arduino-cli config in environment, near CLI binary, project root, or user home
pub fn find_config(worktree: &zed::Worktree, cli_path: Option<&str>) -> Option<String> {
    // 1. Check environment variable override
    if let Some(path) = crate::utils::get_env(worktree, "ARDUINO_CLI_CONFIG") {
        if tools::file_exists(&path) {
            eprintln!("Arduino: Using config from ARDUINO_CLI_CONFIG: {}", path);
            return Some(path);
        }
    }

    // 2. Check standard Arduino directory environment variables
    if let Some(data_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_DATA") {
        let config_path = format!("{}/arduino-cli.yaml", data_dir);
        if tools::file_exists(&config_path) {
            eprintln!(
                "Arduino: Found config in ARDUINO_DIRECTORIES_DATA: {}",
                config_path
            );
            return Some(config_path);
        }
    }

    if let Some(user_dir) = crate::utils::get_env(worktree, "ARDUINO_DIRECTORIES_USER") {
        let config_path = format!("{}/arduino-cli.yaml", user_dir);
        if tools::file_exists(&config_path) {
            eprintln!(
                "Arduino: Found config in ARDUINO_DIRECTORIES_USER: {}",
                config_path
            );
            return Some(config_path);
        }
    }

    // 3. Check near CLI binary
    if let Some(cli) = cli_path {
        use std::path::PathBuf;
        if let Some(parent) = PathBuf::from(cli).parent() {
            let config_near_cli = parent.join("arduino-cli.yaml");
            if tools::file_exists(&config_near_cli.to_string_lossy()) {
                let path = config_near_cli.to_string_lossy().to_string();
                eprintln!("Arduino: Found config near arduino-cli binary: {}", path);
                return Some(path);
            }
        }
    }

    // 4. Check project root
    let project_config = format!("{}/.arduino-cli.yaml", worktree.root_path());
    if tools::file_exists(&project_config) {
        return Some(project_config);
    }

    let project_config_alt = format!("{}/arduino-cli.yaml", worktree.root_path());
    if tools::file_exists(&project_config_alt) {
        return Some(project_config_alt);
    }

    // 5. Check user home directory
    if let Some(home) = crate::utils::get_home(worktree) {
        let home_config = format!("{}/.arduino15/arduino-cli.yaml", home);
        if tools::file_exists(&home_config) {
            return Some(home_config);
        }

        // Windows location
        let windows_config = format!("{}\\Arduino15\\arduino-cli.yaml", home);
        if tools::file_exists(&windows_config) {
            return Some(windows_config);
        }

        // macOS location
        let macos_config = format!("{}/Library/Arduino15/arduino-cli.yaml", home);
        if tools::file_exists(&macos_config) {
            return Some(macos_config);
        }
    }

    None
}

// ============================================================================
// Public API: CLI Wrapper Functions
// ============================================================================

/// Validate FQBN format (vendor:architecture:board or vendor:architecture:board:options)
pub fn validate_fqbn(fqbn: &str) -> Result<()> {
    let parts: Vec<&str> = fqbn.split(':').collect();

    if parts.len() < 3 {
        return Err(format!(
            "Invalid FQBN format: '{}'. Expected format: vendor:architecture:board[:options]",
            fqbn
        ));
    }

    if parts[0].is_empty() || parts[1].is_empty() || parts[2].is_empty() {
        return Err(format!(
            "Invalid FQBN '{}': vendor, architecture, and board cannot be empty",
            fqbn
        ));
    }

    Ok(())
}

/// Extract core ID from FQBN (e.g., "esp32:esp32" from "esp32:esp32:esp32s3")
pub fn extract_core_id(fqbn: &str) -> Option<String> {
    let parts: Vec<&str> = fqbn.split(':').collect();
    if parts.len() >= 2 {
        Some(format!("{}:{}", parts[0], parts[1]))
    } else {
        None
    }
}

/// Check if board core is installed via `arduino-cli core list`
pub fn is_core_installed(cli_path: &str, core_id: &str) -> bool {
    Command::new(cli_path)
        .arg("core")
        .arg("list")
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.starts_with(core_id))
        })
        .unwrap_or(false)
}

/// Install board core (can be slow, cores are 100MB+)
pub fn install_core(cli_path: &str, core_id: &str, config_path: Option<&str>) -> Result<()> {
    eprintln!("Arduino: Installing core {}...", core_id);

    let mut cmd = Command::new(cli_path);
    cmd.arg("core").arg("install").arg(core_id);

    if let Some(config) = config_path {
        cmd.arg("--config-file").arg(config);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run arduino-cli core install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("arduino-cli core install failed: {}", stderr));
    }

    Ok(())
}

/// Generate compile_commands.json for clangd (10-30 seconds)
pub fn generate_compile_db(
    cli_path: &str,
    fqbn: &str,
    config_path: Option<&str>,
    library_paths: &[String],
    worktree: &zed::Worktree,
) -> Result<()> {
    let worktree_root = worktree.root_path();

    let mut cmd = Command::new(cli_path);
    cmd.arg("compile")
        .arg("--fqbn")
        .arg(fqbn)
        .arg("--only-compilation-database")
        .arg(worktree_root);

    if let Some(config) = config_path {
        cmd.arg("--config-file").arg(config);
    }

    if !library_paths.is_empty() {
        cmd.arg("--libraries").arg(library_paths.join(","));
    }

    // Add custom arduino-cli compile arguments from settings
    let custom_args = crate::utils::get_string_array_setting(worktree, "cli.compileArguments");
    for arg in custom_args {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run arduino-cli compile: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("arduino-cli compile failed: {}", stderr));
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_dependency_error(error_msg: &str, auto_download: bool) -> String {
    let recovery_steps = if auto_download {
        vec![
            "1. Check your internet connection and restart Zed",
            "2. Check GitHub API rate limits: https://api.github.com/rate_limit",
            "3. If problems persist, manually install arduino-cli:",
            "   https://arduino.github.io/arduino-cli/latest/installation/",
        ]
    } else {
        vec![
            "1. Enable 'autoDownloadCli' in extension settings, OR",
            "2. Manually install arduino-cli:",
            "   https://arduino.github.io/arduino-cli/latest/installation/",
            "3. Add to PATH or set 'cli.path' in settings",
        ]
    };

    format!(
        "Arduino CLI Error: {}\n\nRecovery options:\n{}",
        error_msg,
        recovery_steps.join("\n")
    )
}
