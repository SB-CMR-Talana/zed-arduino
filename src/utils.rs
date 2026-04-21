use std::fs;
use std::path::Path;
use zed_extension_api::{self as zed, settings::LspSettings, Result};

/// Convert platform/arch to strings for GitHub release asset names
pub fn platform_strings(
    platform: zed::Os,
    arch: zed::Architecture,
) -> (&'static str, &'static str) {
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

/// Get boolean setting from LSP config (returns default if not found)
pub fn get_setting(worktree: &zed::Worktree, key: &str, default: bool) -> bool {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|s| s.settings)
        .and_then(|s| s.get(key).and_then(zed::serde_json::Value::as_bool))
        .unwrap_or(default)
}

/// Get string setting from LSP config (returns default if not found)
pub fn get_string_setting(worktree: &zed::Worktree, key: &str, default: &str) -> String {
    LspSettings::for_worktree("arduino", worktree)
        .ok()
        .and_then(|s| s.settings)
        .and_then(|s| s.get(key).and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| default.to_string())
}

/// Get argument value from command line args (e.g., get value after "-fqbn")
pub fn get_arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()))
}

/// Get home directory from environment (cross-platform: HOME on Unix, USERPROFILE on Windows)
pub fn get_home(worktree: &zed::Worktree) -> Option<String> {
    use std::collections::HashMap;
    let shell_env: HashMap<String, String> = worktree.shell_env().into_iter().collect();
    shell_env
        .get("HOME")
        .or_else(|| shell_env.get("USERPROFILE"))
        .cloned()
}

/// Auto-generate .zed/settings.json with safe defaults if it doesn't exist
/// Controlled by autoGenerateProjectSettings setting (default true)
pub fn auto_generate_project_settings(worktree: &zed::Worktree) -> Result<()> {
    // Check if feature is enabled
    if !get_setting(worktree, "autoGenerateProjectSettings", true) {
        return Ok(());
    }

    let worktree_root = worktree.root_path();
    let zed_dir = format!("{}/.zed", worktree_root);
    let settings_file = format!("{}/settings.json", zed_dir);

    // If settings.json already exists, don't overwrite it
    if Path::new(&settings_file).exists() {
        return Ok(());
    }

    // Create .zed directory if it doesn't exist
    fs::create_dir_all(&zed_dir).map_err(|e| format!("failed to create .zed directory: {}", e))?;

    // Default settings template
    let default_settings = r#"{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": [
          "-fqbn",
          "REPLACE_WITH_YOUR_BOARD_FQBN"
        ]
      },
      "settings": {
        "autoGenerateProjectSettings": true,
        "githubRepo": "arduino/arduino-language-server",
        "autoDownloadCli": true,
        "autoCreateConfig": false,
        "autoInstallCore": false,
        "autoGenerateCompileDb": false
      }
    }
  },
  "languages": {
    "Arduino": {
      "format_on_save": "off",
      "tab_size": 2
    }
  }
}
"#;

    fs::write(&settings_file, default_settings)
        .map_err(|e| format!("failed to write .zed/settings.json: {}", e))?;

    Ok(())
}

/// Check and report missing dependencies and configuration issues
#[allow(dead_code)]
pub fn check_dependencies(worktree: &zed::Worktree) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for arduino-cli
    if worktree.which("arduino-cli").is_none() {
        warnings.push(
            "arduino-cli not found in PATH. It will be auto-downloaded, or install manually with 'brew install arduino-cli'".to_string()
        );
    }

    // Check for clangd
    if crate::detection::find_clangd(worktree).is_none() {
        warnings.push(
            "clangd not found. IntelliSense will be limited. Open a C++ file to trigger Zed's automatic installation, or install manually.".to_string()
        );
    }

    // Check for arduino-cli config
    if crate::detection::find_arduino_cli_config(worktree).is_none() {
        warnings.push(
            "arduino-cli.yaml not found. Some features may be limited. Run 'arduino-cli config init' or enable 'autoCreateConfig'.".to_string()
        );
    }

    // Check for compilation database
    if !crate::detection::check_compilation_database(worktree) {
        warnings.push(
            "compile_commands.json not found. Run 'arduino-cli compile --fqbn YOUR:BOARD:FQBN --only-compilation-database .' or enable 'autoGenerateCompileDb'.".to_string()
        );
    }

    // Check for FQBN in settings
    if let Ok(lsp_settings) =
        zed_extension_api::settings::LspSettings::for_worktree("arduino", worktree)
    {
        if let Some(binary) = lsp_settings.binary {
            if let Some(args) = binary.arguments {
                if !args.iter().any(|arg| arg == "-fqbn") {
                    warnings.push(
                        "FQBN not configured. Add '-fqbn' argument in .zed/settings.json (e.g., 'arduino:avr:uno')".to_string()
                    );
                }
            } else {
                warnings.push(
                    "No binary arguments configured. Add '-fqbn' in .zed/settings.json".to_string(),
                );
            }
        } else {
            warnings.push(
                "No binary configuration found. Add '-fqbn' in .zed/settings.json".to_string(),
            );
        }
    }

    warnings
}
