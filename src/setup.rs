//! Auto-generates Arduino project configuration files and manages arduino-cli isolated setup.

use std::fs;
use std::path::Path;
use zed_extension_api::{self as zed, Result};

use crate::metadata::InstallationState;
use crate::utils::get_setting;

// ============================================================================
// Public Auto-Generation Functions
// ============================================================================

/// Auto-generate .zed/tasks.json with Arduino commands
pub fn auto_generate_tasks(worktree: &zed::Worktree, state: &InstallationState) -> Result<()> {
    // Check if feature is enabled
    if !get_setting(worktree, "autoGenerateTasks", true) {
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
    fs::create_dir_all(&zed_dir).map_err(|e| {
        eprintln!("Arduino: Failed to create .zed directory for tasks: {}", e);
        format!("failed to create .zed directory: {}", e)
    })?;

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

    let fqbn_extract_helper = r#"FQBN=$(grep '\"fqbn\"' .zed/settings.json 2>/dev/null | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ -z \"$FQBN\" ]; then FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json 2>/dev/null | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"'); fi; if [ -z \"$FQBN\" ]; then echo 'Error: FQBN not found in .zed/settings.json'; echo 'Add \"fqbn\": \"arduino:avr:uno\" to lsp.arduino.settings'; exit 1; fi"#;

    // Default tasks template
    let default_tasks = format!(
        r#"{{
  // Arduino Extension Tasks
  // For documentation and customization options, see the extension README:
  // {}
  // Or online: https://github.com/SB-CMR-Talana/zed-arduino
  //
  // To regenerate this file:
  // 1. Run the "Arduino: Regenerate Tasks File" task, or
  // 2. Delete this file and restart Zed (auto-generates), or
  // 3. Delete this file and run any Arduino task (triggers auto-generation)
  "tasks": [
    {{
      "label": "Arduino: Generate Settings File",
      "command": "mkdir -p .zed && arduino-cli board list && echo '' && echo 'Creating settings template in .zed/settings.json' && cat > .zed/settings.json << 'SETTINGS_EOF'
{{
  \"lsp\": {{
    \"arduino\": {{
      \"settings\": {{
        \"fqbn\": \"arduino:avr:uno\",
        \"port\": \"/dev/ttyUSB0\",
        \"baudRate\": 9600,
        \"autoCreateConfig\": true
      }}
    }}
  }}
}}
SETTINGS_EOF
echo '' && echo 'Created .zed/settings.json - Edit the FQBN and port above' && cat .zed/settings.json",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Regenerate Tasks File",
      "command": "echo 'Regenerating .zed/tasks.json...' && rm -f .zed/tasks.json && echo 'Deleted old tasks.json. Restart Zed or run any task to trigger auto-generation.' && echo 'Note: You may need to reload the project (Cmd+Shift+P -> \"zed: reload project\") for changes to take effect.'",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Boards & Ports",
      "command": "arduino-cli board list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Show Board Options",
      "command": "{}; echo '' && echo 'Current FQBN:' && echo \"  $FQBN\" && echo '' && arduino-cli board details -b \"$FQBN\" && echo '' && echo '=== How to Use FQBN Options ===' && echo 'Base format: vendor:architecture:board' && echo 'With options: vendor:architecture:board:option1=value1,option2=value2' && echo '' && echo 'Example for your board:' && BASE_FQBN=$(echo \"$FQBN\" | cut -d: -f1-3) && echo \"  $BASE_FQBN:UploadSpeed=921600,FlashFreq=80\" && echo '' && echo 'Copy option values from the rightmost column above (e.g., UploadSpeed=921600)' && echo 'Combine multiple options with commas and append to base FQBN'",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile",
      "command": "{}; arduino-cli compile -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile (Verbose)",
      "command": "{}; arduino-cli compile -v -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upload (last compile)",
      "command": "{}; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile & Upload",
      "command": "{}; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli compile -b \"$FQBN\" . && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Monitor Serial",
      "command": "PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; BAUD=$(grep '\"baudRate\"' .zed/settings.json | grep -o '[0-9]\\+' | head -1); if [ -z \"$BAUD\" ]; then BAUD=9600; fi; arduino-cli monitor -p \"$PORT\" --config \"$BAUD\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Show Sketch Size",
      "command": "{}; arduino-cli compile -b \"$FQBN\" .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Generate Compilation Database",
      "command": "{}; arduino-cli compile --fqbn \"$FQBN\" --only-compilation-database .",
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
      "command": "{}; arduino-cli board details -b \"$FQBN\"",
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
      "label": "Arduino: Show Detected Tools",
      "command": "echo '=== Arduino Extension Tool Detection ===' && echo '' && echo 'Checking for clangd...' && (command -v clangd >/dev/null 2>&1 && echo \"  Found in PATH: $(command -v clangd)\" && clangd --version 2>&1 | head -1 || echo '  Not found in PATH') && ([ -f ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd ] && echo \"  Found Zed Flatpak: $(ls ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd 2>/dev/null | head -1)\" && $(ls ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd 2>/dev/null | head -1) --version 2>&1 | head -1 || echo '  Zed Flatpak: not found') && echo '' && echo 'Checking for arduino-cli...' && (command -v arduino-cli >/dev/null 2>&1 && echo \"  Found in PATH: $(command -v arduino-cli)\" && arduino-cli version 2>&1 | head -1 || echo '  Not found in PATH') && ([ -f ~/.arduino15/arduino-cli ] && echo '  Found in ~/.arduino15/' || echo '  ~/.arduino15/: not found') && echo '' && echo 'Checking for arduino-cli.yaml config...' && ([ -f ~/.arduino15/arduino-cli.yaml ] && echo '  Found: ~/.arduino15/arduino-cli.yaml' || echo '  Not found in ~/.arduino15/') && ([ -f .arduino-cli.yaml ] && echo '  Found: ./.arduino-cli.yaml' || echo '  Not found in project root') && echo '' && echo 'Environment variables:' && ([ -n \"$CLANGD_PATH\" ] && echo \"  CLANGD_PATH=$CLANGD_PATH\" || echo '  CLANGD_PATH: not set') && ([ -n \"$ARDUINO_CLI_PATH\" ] && echo \"  ARDUINO_CLI_PATH=$ARDUINO_CLI_PATH\" || echo '  ARDUINO_CLI_PATH: not set') && ([ -n \"$ARDUINO_CLI_CONFIG\" ] && echo \"  ARDUINO_CLI_CONFIG=$ARDUINO_CLI_CONFIG\" || echo '  ARDUINO_CLI_CONFIG: not set') && ([ -n \"$ARDUINO_DIRECTORIES_DATA\" ] && echo \"  ARDUINO_DIRECTORIES_DATA=$ARDUINO_DIRECTORIES_DATA\" || echo '  ARDUINO_DIRECTORIES_DATA: not set') && ([ -n \"$ARDUINO_DIRECTORIES_USER\" ] && echo \"  ARDUINO_DIRECTORIES_USER=$ARDUINO_DIRECTORIES_USER\" || echo '  ARDUINO_DIRECTORIES_USER: not set') && echo '' && echo 'Note: The extension checks these locations in priority order:' && echo '  1. Explicit settings (binary.arguments or settings.*)' && echo '  2. Environment variables' && echo '  3. PATH' && echo '  4. Tool-specific locations (shown above)' && echo '  5. Download if not found'",
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
        readme_path,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        clear_clangd_cmd,
        clear_arduino_cli_cmd
    );

    fs::write(&tasks_file, default_tasks).map_err(|e| {
        eprintln!("Arduino: Failed to write .zed/tasks.json: {}", e);
        format!("failed to write .zed/tasks.json: {}", e)
    })?;

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

// ============================================================================
// Helpers
// ============================================================================

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
