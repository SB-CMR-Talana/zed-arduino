use std::fs;
use std::path::Path;
use zed_extension_api::{self as zed, Result};

use crate::metadata::InstallationState;
use crate::utils::get_setting;

fn get_extension_readme_path() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.join("README.md").to_str().map(String::from))
        .unwrap_or_else(|| {
            let (platform, _) = zed::current_platform();
            match platform {
                zed::Os::Linux => "~/.local/share/zed/extensions/arduino/README.md".to_string(),
                zed::Os::Mac => {
                    "~/Library/Application Support/Zed/extensions/arduino/README.md".to_string()
                }
                zed::Os::Windows => "%APPDATA%\\Zed\\extensions\\arduino\\README.md".to_string(),
            }
        })
}

fn get_default_board_settings(worktree: &zed::Worktree) -> (String, String) {
    const DEFAULT_FQBN: &str = "REPLACE_WITH_YOUR_BOARD_FQBN";
    const DEFAULT_PORT: &str = "REPLACE_WITH_YOUR_PORT";

    worktree
        .which("arduino-cli")
        .and_then(|cli_path| crate::cli::detect_connected_board(&cli_path))
        .map(|(port, fqbn)| {
            eprintln!("Arduino: Detected board {} on port {}", fqbn, port);
            (fqbn, port)
        })
        .unwrap_or_else(|| (DEFAULT_FQBN.to_string(), DEFAULT_PORT.to_string()))
}

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
    let (default_fqbn, default_port) = get_default_board_settings(worktree);

    let readme_path = get_extension_readme_path();

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
        "port": "{}",
        "libraryPaths": []
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
pub fn auto_generate_tasks(worktree: &zed::Worktree, state: &InstallationState) -> Result<()> {
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

    // Determine platform-specific cache clear commands
    let (clear_clangd_cmd, clear_arduino_cli_cmd) = match state.get_platform() {
        Some(crate::metadata::Platform::Windows) => (
            r#"echo 'Clearing clangd cache...' && (if exist .cache\clangd rmdir /s /q .cache\clangd) && (if exist %LOCALAPPDATA%\clangd\cache rmdir /s /q %LOCALAPPDATA%\clangd\cache) && echo 'clangd cache cleared'"#,
            r#"echo 'Clearing arduino-cli cache...' && if exist %LOCALAPPDATA%\arduino-cli\cache rmdir /s /q %LOCALAPPDATA%\arduino-cli\cache && echo 'arduino-cli cache cleared'"#,
        ),
        _ => (
            // Linux/macOS
            r#"echo 'Clearing clangd cache...' && rm -rf .cache/clangd/ ~/.cache/clangd/ && echo 'clangd cache cleared'"#,
            r#"echo 'Clearing arduino-cli cache...' && rm -rf ~/.cache/arduino-cli/ ~/Library/Caches/arduino-cli/ && echo 'arduino-cli cache cleared'"#,
        ),
    };

    let readme_path = get_extension_readme_path();

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
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; arduino-cli compile -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile (Verbose)",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; arduino-cli compile -v -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upload (last compile)",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile & Upload",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli compile -b \"$FQBN\" . && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Monitor Serial",
      "command": "PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli monitor -p \"$PORT\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Show Sketch Size",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; arduino-cli compile -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Generate Compilation Database",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; arduino-cli compile --fqbn \"$FQBN\" --only-compilation-database .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Update Core Index",
      "command": "arduino-cli core update-index",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Search Boards",
      "command": "echo 'Enter search term:' && read SEARCH && arduino-cli board listall | grep -i \"$SEARCH\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Installed Cores",
      "command": "arduino-cli core list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Install Core",
      "command": "arduino-cli core list && echo '' && echo 'Enter core to install (e.g., arduino:avr):' && read CORE && arduino-cli core install \"$CORE\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Uninstall Core",
      "command": "arduino-cli core list && echo '' && echo 'Enter core to uninstall (e.g., arduino:avr):' && read CORE && arduino-cli core uninstall \"$CORE\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upgrade All Cores",
      "command": "arduino-cli core upgrade",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Board Details",
      "command": "FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"') || {{ echo 'Error: FQBN not found in .zed/settings.json'; exit 1; }}; arduino-cli board details -b \"$FQBN\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Search Libraries",
      "command": "echo 'Enter search term:' && read SEARCH && arduino-cli lib search \"$SEARCH\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Installed Libraries",
      "command": "arduino-cli lib list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Install Library",
      "command": "arduino-cli lib list && echo '' && echo 'Enter library name to install:' && read LIBRARY && arduino-cli lib install \"$LIBRARY\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Uninstall Library",
      "command": "arduino-cli lib list && echo '' && echo 'Enter library name to uninstall:' && read LIBRARY && arduino-cli lib uninstall \"$LIBRARY\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upgrade All Libraries",
      "command": "arduino-cli lib upgrade",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Show Library Dependencies",
      "command": "echo 'Enter library name:' && read LIBRARY && arduino-cli lib deps \"$LIBRARY\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Examples",
      "command": "arduino-cli lib examples",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Show Extension Status",
      "command": "if [ -f installation_state.json ]; then echo 'Extension Installation Status:' && cat installation_state.json | grep -v '{{' | grep -v '}}' || cat installation_state.json; else echo 'No installation state found. Tools may be using system installations.'; fi",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Clear clangd Cache",
      "command": "{}",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Clear arduino-cli Cache",
      "command": "{}",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Clean Build",
      "command": "rm -rf build compile_commands.json *.elf *.hex *.bin && echo 'Build artifacts cleaned'",
      "use_new_terminal": false
    }}
  ]
}}
"#,
        readme_path, clear_clangd_cmd, clear_arduino_cli_cmd
    );

    fs::write(&tasks_file, default_tasks)
        .map_err(|e| format!("failed to write .zed/tasks.json: {}", e))?;

    Ok(())
}

/// Create isolated arduino-cli config file that stores all data in extension directory
pub fn create_isolated_arduino_config(state: &InstallationState) -> Result<()> {
    // Only create if arduino-cli was downloaded by the extension
    if !state.arduino_cli_installed_by_extension() {
        return Ok(());
    }

    let config_file = "arduino-cli-isolated.yaml";
    let data_dir = state
        .get_arduino_cli_data_dir()
        .unwrap_or_else(|| "arduino-data".to_string());

    // Create isolated config pointing all directories to extension work directory
    let config_content = format!(
        r#"# Arduino CLI isolated configuration (managed by Zed extension)
# All data is stored in the extension's directory for clean isolation

directories:
  data: {}
  downloads: {}/staging
  user: {}/user
  builtin_tools: {}/builtin_tools

daemon:
  port: "50051"

output:
  no_color: false

board_manager:
  additional_urls: []

library:
  enable_unsafe_install: false

logging:
  level: info
  format: text

updater:
  enable_notification: false

# Note: Cache directory cannot be configured in arduino-cli config.
# On Linux: May use ~/.cache/arduino-cli/
# On macOS: May use ~/Library/Caches/arduino-cli/
# On Windows: May use %LOCALAPPDATA%\arduino-cli\
# These cache files are typically small and temporary.
"#,
        data_dir, data_dir, data_dir, data_dir
    );

    // Create data directory if it doesn't exist
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("failed to create arduino data directory: {}", e))?;

    // Write config file
    fs::write(config_file, config_content)
        .map_err(|e| format!("failed to write isolated arduino-cli config: {}", e))?;

    eprintln!(
        "Arduino: Created isolated configuration - all cores and libraries will be stored in extension directory"
    );

    Ok(())
}
