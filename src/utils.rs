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
