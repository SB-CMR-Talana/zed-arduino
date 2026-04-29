//! Validation utilities for checking binary existence, functionality, and dependencies.

use std::fs;
use std::process::Command;
use zed_extension_api::{self as zed};

// ============================================================================
// Binary Validators
// ============================================================================

/// Validate that a binary exists and is executable
pub fn validate_binary_exists(path: &str) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("Binary not found at '{}': {}", path, e))?;

    if !metadata.is_file() {
        return Err(format!("Path '{}' exists but is not a file", path));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(format!("Binary '{}' exists but is not executable", path));
        }
    }

    Ok(())
}

/// Validate arduino-cli by running version command
pub fn validate_arduino_cli(path: &str) -> Result<String, String> {
    validate_binary_exists(path)?;

    let output = Command::new(path)
        .arg("version")
        .output()
        .map_err(|e| format!("Failed to execute arduino-cli at '{}': {}. The binary may be corrupted or incompatible with your system.", path, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("arduino-cli validation failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .next()
        .unwrap_or("version unknown")
        .to_string())
}

/// Validate clangd by running version command
pub fn validate_clangd(path: &str) -> Result<String, String> {
    validate_binary_exists(path)?;

    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute clangd at '{}': {}. The binary may be corrupted or incompatible with your system.", path, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("clangd validation failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .next()
        .unwrap_or("version unknown")
        .to_string())
}

/// Validate arduino-language-server by running version command
pub fn validate_language_server(path: &str) -> Result<String, String> {
    validate_binary_exists(path)?;

    let output = Command::new(path)
        .arg("-version")
        .output()
        .map_err(|e| format!("Failed to execute arduino-language-server at '{}': {}. The binary may be corrupted or incompatible with your system.", path, e))?;

    if !output.status.success() {
        // Language server might not support -version, just verify it exists
        return Ok("installed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .next()
        .unwrap_or("version unknown")
        .to_string())
}

// ============================================================================
// Dependency Checks
// ============================================================================

/// Check for missing dependencies and configuration issues. Returns (errors, warnings).
pub fn check_dependencies(worktree: &zed::Worktree) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let auto_download_cli = crate::utils::get_setting(worktree, "autoDownloadCli", true);
    let auto_create_config = crate::utils::get_setting(worktree, "autoCreateConfig", true);
    let auto_generate_compile_db =
        crate::utils::get_setting(worktree, "autoGenerateCompileDb", true);

    // Check for arduino-cli
    if worktree.which("arduino-cli").is_none() {
        if auto_download_cli {
            warnings.push(
                "arduino-cli not found in PATH. It will be auto-downloaded on first use."
                    .to_string(),
            );
        } else {
            errors.push(
                "arduino-cli not found in PATH and auto-download is disabled.\n  \
                Install with: brew install arduino-cli (macOS) or see https://arduino.github.io/arduino-cli/installation/\n  \
                Or enable auto-download in settings: \"autoDownloadCli\": true"
                    .to_string(),
            );
        }
    }

    // Check for clangd (optional but recommended)
    if crate::detection::find_clangd(worktree).is_none() {
        warnings.push(
            "clangd not found. IntelliSense features will be limited.\n  \
            Recommended: Open any C++ file in Zed to trigger automatic clangd installation.\n  \
            Or install manually: brew install llvm (macOS) or apt install clangd (Linux)"
                .to_string(),
        );
    }

    // Check for arduino-cli config (optional)
    if crate::detection::find_arduino_cli_config(worktree, None).is_none() {
        if auto_create_config {
            warnings.push(
                "arduino-cli.yaml not found. A minimal config will be auto-created.".to_string(),
            );
        } else {
            warnings.push(
                "arduino-cli.yaml not found. Some features may be limited.\n  \
                Run 'arduino-cli config init' or enable 'autoCreateConfig' in settings."
                    .to_string(),
            );
        }
    }

    // Check for compilation database (optional but recommended)
    if !crate::detection::check_compilation_database(worktree) {
        if auto_generate_compile_db {
            warnings.push(
                "compile_commands.json not found. It will be auto-generated on first compile."
                    .to_string(),
            );
        } else {
            warnings.push(
                "compile_commands.json not found. IntelliSense accuracy will be limited.\n  \
                Generate with task 'Arduino: Generate Compilation Database' or enable 'autoGenerateCompileDb' in settings."
                    .to_string(),
            );
        }
    }

    // Check for FQBN configuration (critical)
    let mut fqbn_configured = false;

    // First check settings.fqbn
    let fqbn_in_settings = crate::utils::get_string_setting(worktree, "fqbn", "");
    if !fqbn_in_settings.is_empty() {
        fqbn_configured = true;
    }

    // Fall back to binary.arguments (backward compatibility)
    if !fqbn_configured {
        if let Ok(lsp_settings) =
            zed_extension_api::settings::LspSettings::for_worktree("arduino", worktree)
        {
            if let Some(binary) = lsp_settings.binary {
                if let Some(args) = binary.arguments {
                    fqbn_configured = args.iter().any(|arg| arg == "-fqbn");
                }
            }
        }
    }

    if !fqbn_configured {
        errors.push(
            "FQBN not configured. The extension cannot function without it.\n  \
            Add to .zed/settings.json:\n  \
            \"lsp\": { \"arduino\": { \"settings\": { \"fqbn\": \"arduino:avr:uno\" } } }\n  \
            Or use binary.arguments (legacy): { \"binary\": { \"arguments\": [\"-fqbn\", \"arduino:avr:uno\"] } }\n  \
            Find your board's FQBN with task 'Arduino: List Boards & Ports'"
                .to_string(),
        );
    }

    (errors, warnings)
}

/// Print dependency check results to stderr
pub fn report_dependencies(worktree: &zed::Worktree) {
    let (errors, warnings) = check_dependencies(worktree);

    if !errors.is_empty() {
        eprintln!("\n❌ Arduino Extension - Critical Issues:");
        for error in &errors {
            eprintln!("\n{}", error);
        }
    }

    if !warnings.is_empty() {
        eprintln!("\n⚠️  Arduino Extension - Recommendations:");
        for warning in &warnings {
            eprintln!("\n{}", warning);
        }
    }

    if errors.is_empty() && warnings.is_empty() {
        eprintln!("✅ Arduino Extension - All dependencies configured");
    }
}

// ============================================================================
// Error Formatting
// ============================================================================

/// Format a dependency error with tool name, reason, and recovery steps
pub fn format_dependency_error(
    tool_name: &str,
    error: &str,
    auto_download_enabled: bool,
) -> String {
    let header = format!(
        "Arduino Extension Error: Failed to obtain {}\n  Reason: {}\n\nRecovery Options:\n",
        tool_name, error
    );

    let recovery_steps = match tool_name {
        "arduino-cli" => get_arduino_cli_recovery_steps(auto_download_enabled),
        "clangd" => get_clangd_recovery_steps(),
        "arduino-language-server" => get_language_server_recovery_steps(auto_download_enabled),
        _ => vec![
            "1. Check the extension logs for more details",
            "2. Restart Zed and try again",
        ],
    };

    let footer = "\nFor more help, see: https://github.com/itzderock/zed-arduino#troubleshooting\n";

    format!(
        "{}{}{}",
        header,
        recovery_steps
            .iter()
            .map(|s| format!("  {}\n", s))
            .collect::<String>(),
        footer
    )
}

fn get_arduino_cli_recovery_steps(auto_download_enabled: bool) -> Vec<&'static str> {
    if auto_download_enabled {
        vec![
            "1. Check your internet connection and restart Zed",
            "2. Install manually: https://arduino.github.io/arduino-cli/installation/",
            "   - macOS: brew install arduino-cli",
            "   - Linux: curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh",
            "   - Windows: Download from https://github.com/arduino/arduino-cli/releases",
            "3. Disable auto-download and specify path in settings:",
            "   \"autoDownloadCli\": false,",
            "   \"binary\": { \"path\": \"/path/to/arduino-cli\" }",
        ]
    } else {
        vec![
            "1. Install arduino-cli: https://arduino.github.io/arduino-cli/installation/",
            "   - macOS: brew install arduino-cli",
            "   - Linux: curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh",
            "   - Windows: Download from https://github.com/arduino/arduino-cli/releases",
            "2. Ensure arduino-cli is in your PATH",
            "3. Or enable auto-download in settings: \"autoDownloadCli\": true",
        ]
    }
}

fn get_clangd_recovery_steps() -> Vec<&'static str> {
    vec![
        "1. Open any C++ file in Zed to trigger automatic clangd installation",
        "2. Install manually:",
        "   - macOS: brew install llvm",
        "   - Linux: apt install clangd (or equivalent for your distro)",
        "   - Windows: Download from https://github.com/clangd/clangd/releases",
        "3. IntelliSense will be limited without clangd, but basic features will work",
    ]
}

fn get_language_server_recovery_steps(auto_download_enabled: bool) -> Vec<&'static str> {
    if auto_download_enabled {
        vec![
            "1. Check your internet connection and restart Zed",
            "2. Check GitHub API rate limits: https://api.github.com/rate_limit",
            "3. Try using a custom fork in settings:",
            "   \"ls\": { \"githubRepo\": \"arduino/arduino-language-server\" }",
        ]
    } else {
        vec![
            "1. Download from: https://github.com/arduino/arduino-language-server/releases",
            "2. Specify path in settings:",
            "   \"binary\": { \"path\": \"/path/to/arduino-language-server\" }",
        ]
    }
}
