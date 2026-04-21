use std::fs;
use std::process::Command;
use zed_extension_api::{self as zed};

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

/// Validate that arduino-cli works by running version command
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

/// Validate that clangd works by running version command
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

/// Validate that arduino-language-server works by running version command
pub fn validate_language_server(path: &str) -> Result<String, String> {
    validate_binary_exists(path)?;

    let output = Command::new(path)
        .arg("-version")
        .output()
        .map_err(|e| format!("Failed to execute arduino-language-server at '{}': {}. The binary may be corrupted or incompatible with your system.", path, e))?;

    if !output.status.success() {
        // Language server might not support -version, so just check if it exists and is executable
        return Ok("installed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .next()
        .unwrap_or("version unknown")
        .to_string())
}

/// Format a user-friendly error message with recovery suggestions
pub fn format_dependency_error(
    tool_name: &str,
    error: &str,
    auto_download_enabled: bool,
) -> String {
    let mut message = format!("Arduino Extension Error: Failed to obtain {}\n", tool_name);
    message.push_str(&format!("  Reason: {}\n\n", error));
    message.push_str("Recovery Options:\n");

    match tool_name {
        "arduino-cli" => {
            if auto_download_enabled {
                message.push_str("  1. Check your internet connection and restart Zed\n");
                message.push_str(
                    "  2. Install manually: https://arduino.github.io/arduino-cli/installation/\n",
                );
                message.push_str("     - macOS: brew install arduino-cli\n");
                message.push_str("     - Linux: curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh\n");
                message.push_str("     - Windows: Download from https://github.com/arduino/arduino-cli/releases\n");
                message.push_str("  3. Disable auto-download and specify path in settings:\n");
                message.push_str("     \"autoDownloadCli\": false,\n");
                message.push_str("     \"binary\": { \"path\": \"/path/to/arduino-cli\" }\n");
            } else {
                message.push_str("  1. Install arduino-cli: https://arduino.github.io/arduino-cli/installation/\n");
                message.push_str("     - macOS: brew install arduino-cli\n");
                message.push_str("     - Linux: curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh\n");
                message.push_str("     - Windows: Download from https://github.com/arduino/arduino-cli/releases\n");
                message.push_str("  2. Ensure arduino-cli is in your PATH\n");
                message.push_str(
                    "  3. Or enable auto-download in settings: \"autoDownloadCli\": true\n",
                );
            }
        }
        "clangd" => {
            message.push_str(
                "  1. Open any C++ file in Zed to trigger automatic clangd installation\n",
            );
            message.push_str("  2. Install manually:\n");
            message.push_str("     - macOS: brew install llvm\n");
            message.push_str("     - Linux: apt install clangd (or equivalent for your distro)\n");
            message.push_str(
                "     - Windows: Download from https://github.com/clangd/clangd/releases\n",
            );
            message.push_str(
                "  3. IntelliSense will be limited without clangd, but basic features will work\n",
            );
        }
        "arduino-language-server" => {
            if auto_download_enabled {
                message.push_str("  1. Check your internet connection and restart Zed\n");
                message.push_str(
                    "  2. Check GitHub API rate limits: https://api.github.com/rate_limit\n",
                );
                message.push_str("  3. Try using a custom fork in settings:\n");
                message.push_str("     \"githubRepo\": \"arduino/arduino-language-server\"\n");
            } else {
                message.push_str("  1. Download from: https://github.com/arduino/arduino-language-server/releases\n");
                message.push_str("  2. Specify path in settings:\n");
                message.push_str(
                    "     \"binary\": { \"path\": \"/path/to/arduino-language-server\" }\n",
                );
            }
        }
        _ => {
            message.push_str("  1. Check the extension logs for more details\n");
            message.push_str("  2. Restart Zed and try again\n");
        }
    }

    message.push_str(
        "\nFor more help, see: https://github.com/itzderock/zed-arduino#troubleshooting\n",
    );
    message
}

/// Check and report missing dependencies and configuration issues
/// Returns a tuple of (errors, warnings) where errors are critical and warnings are optional
pub fn check_dependencies(worktree: &zed::Worktree) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let auto_download_cli = crate::utils::get_setting(worktree, "autoDownloadCli", true);
    let auto_create_config = crate::utils::get_setting(worktree, "autoCreateConfig", false);
    let auto_generate_compile_db =
        crate::utils::get_setting(worktree, "autoGenerateCompileDb", false);

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
    if crate::detection::find_arduino_cli_config(worktree).is_none() {
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

    // Check for compilation database (optional but recommended for full IntelliSense)
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

    // Check for FQBN in settings (critical)
    let mut fqbn_configured = false;
    if let Ok(lsp_settings) =
        zed_extension_api::settings::LspSettings::for_worktree("arduino", worktree)
    {
        if let Some(binary) = lsp_settings.binary {
            if let Some(args) = binary.arguments {
                fqbn_configured = args.iter().any(|arg| arg == "-fqbn");
            }
        }
    }

    if !fqbn_configured {
        errors.push(
            "FQBN not configured. The extension cannot function without it.\n  \
            Add to .zed/settings.json:\n  \
            \"lsp\": { \"arduino\": { \"binary\": { \"arguments\": [\"-fqbn\", \"arduino:avr:uno\"] } } }\n  \
            Find your board's FQBN with task 'Arduino: List Boards & Ports'"
                .to_string(),
        );
    }

    (errors, warnings)
}

/// Print dependency check results to stderr in a user-friendly format
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
