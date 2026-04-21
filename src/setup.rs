use std::fs;
use std::path::Path;
use zed_extension_api::{self as zed, Result};

use crate::utils::get_setting;

/// Auto-generate .zed/settings.json with default Arduino configuration
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

    // Try to detect connected board and use its values as defaults
    let (default_fqbn, default_port) = if let Some(cli_path) = worktree.which("arduino-cli") {
        if let Some((port, fqbn)) = crate::cli::detect_connected_board(&cli_path) {
            eprintln!("Arduino: Detected board {} on port {}", fqbn, port);
            (fqbn, port)
        } else {
            (
                "REPLACE_WITH_YOUR_BOARD_FQBN".to_string(),
                "REPLACE_WITH_YOUR_PORT".to_string(),
            )
        }
    } else {
        (
            "REPLACE_WITH_YOUR_BOARD_FQBN".to_string(),
            "REPLACE_WITH_YOUR_PORT".to_string(),
        )
    };

    // Get actual extension installation path
    let readme_path = std::env::current_dir()
        .ok()
        .and_then(|p| p.join("README.md").to_str().map(String::from))
        .unwrap_or_else(|| {
            // Fallback to OS-specific default path
            let (platform, _) = zed::current_platform();
            match platform {
                zed::Os::Linux => "~/.local/share/zed/extensions/arduino/README.md".to_string(),
                zed::Os::Mac => {
                    "~/Library/Application Support/Zed/extensions/arduino/README.md".to_string()
                }
                zed::Os::Windows => "%APPDATA%\\Zed\\extensions\\arduino\\README.md".to_string(),
            }
        });

    // Generate settings with detected or default values
    let default_settings = format!(
        r#"{{
  // For documentation and all configuration options, see the extension README:
  //   {}
  // Or online: https://github.com/itzderock/zed-arduino/blob/main/README.md
  "lsp": {{
    "arduino": {{
      "binary": {{
        "arguments": [
          "-fqbn",
          "{}"
        ]
      }},
      "settings": {{
        "autoGenerateProjectSettings": true,
        "githubRepo": "arduino/arduino-language-server",
        "autoDownloadCli": true,
        "autoCreateConfig": false,
        "autoInstallCore": false,
        "autoGenerateCompileDb": false,
        "port": "{}"
      }}
    }}
  }},
  "languages": {{
    "Arduino": {{
      "format_on_save": "off",
      "tab_size": 2
    }}
  }}
}}
"#,
        readme_path, default_fqbn, default_port
    );

    fs::write(&settings_file, default_settings)
        .map_err(|e| format!("failed to write .zed/settings.json: {}", e))?;

    Ok(())
}

/// Auto-generate .zed/tasks.json with Arduino commands
pub fn auto_generate_tasks(worktree: &zed::Worktree) -> Result<()> {
    // Check if feature is enabled
    if !get_setting(worktree, "autoGenerateProjectSettings", true) {
        return Ok(());
    }

    let worktree_root = worktree.root_path();
    let zed_dir = format!("{}/.zed", worktree_root);
    let tasks_file = format!("{}/tasks.json", zed_dir);

    // If tasks.json already exists, don't overwrite it
    if Path::new(&tasks_file).exists() {
        return Ok(());
    }

    // Create .zed directory if it doesn't exist
    fs::create_dir_all(&zed_dir).map_err(|e| format!("failed to create .zed directory: {}", e))?;

    // Get actual extension installation path
    let readme_path = std::env::current_dir()
        .ok()
        .and_then(|p| p.join("README.md").to_str().map(String::from))
        .unwrap_or_else(|| {
            // Fallback to OS-specific default path
            let (platform, _) = zed::current_platform();
            match platform {
                zed::Os::Linux => "~/.local/share/zed/extensions/arduino/README.md".to_string(),
                zed::Os::Mac => {
                    "~/Library/Application Support/Zed/extensions/arduino/README.md".to_string()
                }
                zed::Os::Windows => "%APPDATA%\\Zed\\extensions\\arduino\\README.md".to_string(),
            }
        });

    // Default tasks template
    // Note: Tasks extract FQBN from .zed/settings.json automatically
    let default_tasks = format!(
        r#"{{
  // For documentation and customization options, see the extension README:
  // {}
  // Or online: https://github.com/SB-CMR-Talana/zed-arduino
  "tasks": [
    {{
      "label": "Arduino: List Boards & Ports",
      "command": "arduino-cli board list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') && arduino-cli compile -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upload",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') && PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"') && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile & Upload",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') && PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"') && arduino-cli compile -b \"$FQBN\" . && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Monitor Serial",
      "command": "PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"') && arduino-cli monitor -p \"$PORT\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Clean Build",
      "command": "rm -rf build",
      "use_new_terminal": false
    }}
  ]
}}
"#,
        readme_path
    );

    fs::write(&tasks_file, default_tasks)
        .map_err(|e| format!("failed to write .zed/tasks.json: {}", e))?;

    Ok(())
}
