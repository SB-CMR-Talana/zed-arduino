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

    // Default tasks template
    let default_tasks = r#"{
  "tasks": [
    {
      "label": "Arduino: Compile",
      "command": "arduino-cli compile -b $ZED_ARDUINO_FQBN .",
      "use_new_terminal": false
    },
    {
      "label": "Arduino: Upload",
      "command": "arduino-cli upload -p $ZED_ARDUINO_PORT -b $ZED_ARDUINO_FQBN .",
      "use_new_terminal": false
    },
    {
      "label": "Arduino: Compile & Upload",
      "command": "arduino-cli compile -b $ZED_ARDUINO_FQBN . && arduino-cli upload -p $ZED_ARDUINO_PORT -b $ZED_ARDUINO_FQBN .",
      "use_new_terminal": false
    },
    {
      "label": "Arduino: Monitor Serial",
      "command": "arduino-cli monitor -p $ZED_ARDUINO_PORT",
      "use_new_terminal": true
    },
    {
      "label": "Arduino: Clean Build",
      "command": "rm -rf build",
      "use_new_terminal": false
    }
  ],
  "env": {
    "ZED_ARDUINO_FQBN": "REPLACE_WITH_YOUR_BOARD_FQBN",
    "ZED_ARDUINO_PORT": "/dev/ttyUSB0"
  }
}
"#;

    fs::write(&tasks_file, default_tasks)
        .map_err(|e| format!("failed to write .zed/tasks.json: {}", e))?;

    Ok(())
}
