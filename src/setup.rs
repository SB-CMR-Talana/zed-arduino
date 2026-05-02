//! Auto-generates Arduino project configuration, manages arduino-cli isolated setup, and validates dependencies.

use std::fs;
use std::path::Path;
use zed_extension_api::{self as zed, Result};

use crate::metadata::InstallationState;
use crate::utils::get_setting;

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
    if crate::clangd::find(worktree).is_none() {
        warnings.push(
            "clangd not found. IntelliSense features will be limited.\n  \
            Recommended: Open any C++ file in Zed to trigger automatic clangd installation.\n  \
            Or install manually: brew install llvm (macOS) or apt install clangd (Linux)"
                .to_string(),
        );
    }

    // Check for arduino-cli config (optional)
    if crate::arduino_cli::find_config(worktree, None).is_none() {
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
    if !crate::sketches::check_compilation_database(worktree) {
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

    let readme_path = get_extension_readme_path();

    // Generate platform-specific tasks
    let default_tasks = match state.get_platform() {
        Some(crate::metadata::Platform::Windows) => generate_windows_tasks(&readme_path),
        _ => generate_unix_tasks(&readme_path),
    };

    fs::write(&tasks_file, default_tasks).map_err(|e| {
        eprintln!("Arduino: Failed to write .zed/tasks.json: {}", e);
        format!("failed to write .zed/tasks.json: {}", e)
    })?;

    Ok(())
}

fn generate_unix_tasks(readme_path: &str) -> String {
    let fqbn_extract_helper = r#"FQBN=$(grep '\"fqbn\"' .zed/settings.json 2>/dev/null | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ -z \"$FQBN\" ]; then FQBN=$(grep -A 1 '\"-fqbn\"' .zed/settings.json 2>/dev/null | tail -1 | grep -o '\"[^\"]*\"' | tr -d '\"'); fi; if [ -z \"$FQBN\" ]; then echo 'Error: FQBN not found in .zed/settings.json'; echo 'Add \"fqbn\": \"arduino:avr:uno\" to lsp.arduino.settings'; exit 1; fi\"#;

    // Helper to extract compile arguments from settings
    let compile_args_helper = r#"COMPILE_ARGS=$(grep -A 10 '\"compileArguments\"' .zed/settings.json 2>/dev/null | grep -o '\"[^\"]*\"' | grep -v 'compileArguments' | tr '\n' ' ' | xargs)\"#;

    // Helper to extract upload arguments from settings
    let upload_args_helper = r#"UPLOAD_ARGS=$(grep -A 10 '\"uploadArguments\"' .zed/settings.json 2>/dev/null | grep -o '\"[^\"]*\"' | grep -v 'uploadArguments' | tr '\n' ' ' | xargs)\"#;

    format!(
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
    // === Essential Workflow ===
    // (Most frequently used tasks)

    {{
      "label": "Arduino: Compile, Upload & Monitor",
      "command": "{}; {}; {}; mkdir -p .zed; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; BAUD=$(grep '\"baudRate\"' .zed/settings.json | grep -o '[0-9]\\+' | head -1); if [ -z \"$BAUD\" ]; then BAUD=9600; fi; arduino-cli compile -b \"$FQBN\" $COMPILE_ARGS . 2>&1 | tee .zed/last_compile.log && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" $UPLOAD_ARGS . && arduino-cli monitor -p \"$PORT\" --config \"$BAUD\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile & Upload",
      "command": "{}; {}; {}; mkdir -p .zed; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli compile -b \"$FQBN\" $COMPILE_ARGS . 2>&1 | tee .zed/last_compile.log && arduino-cli upload -p \"$PORT\" -b \"$FQBN\" $UPLOAD_ARGS .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upload (last compile)",
      "command": "{}; {}; PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; arduino-cli upload -p \"$PORT\" -b \"$FQBN\" $UPLOAD_ARGS .",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile",
      "command": "{}; {}; mkdir -p .zed; arduino-cli compile -b \"$FQBN\" $COMPILE_ARGS . 2>&1 | tee .zed/last_compile.log",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Monitor Serial",
      "command": "PORT=$(grep '\"port\"' .zed/settings.json | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ \"$PORT\" = \"REPLACE_WITH_YOUR_PORT\" ]; then PORT=$(arduino-cli board list --format json 2>/dev/null | grep -o '\"address\":\"[^\"]*\"' | head -1 | cut -d'\"' -f4); fi; if [ -z \"$PORT\" ]; then echo 'Error: Port not configured and auto-detection failed'; exit 1; fi; BAUD=$(grep '\"baudRate\"' .zed/settings.json | grep -o '[0-9]\\+' | head -1); if [ -z \"$BAUD\" ]; then BAUD=9600; fi; arduino-cli monitor -p \"$PORT\" --config \"$BAUD\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Boards & Ports",
      "command": "arduino-cli board list",
      "use_new_terminal": true
    }},

    // === Board & Hardware Setup ===

    {{
      "label": "Arduino: Search Boards",
      "command": "echo 'Enter search term:' && read SEARCH && arduino-cli board listall | grep -i \"$SEARCH\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Board Details",
      "command": "{}"; arduino-cli board details -b \"$FQBN\"",
      "use_new_terminal": true
    }},

    // === Core Management ===
    // (Arduino board core installation and updates)

    {{
      "label": "Arduino: Update Core Index",
      "command": "arduino-cli core update-index",
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

    // === Library Management ===
    // (Arduino library installation and updates)

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

    // === Project Management ===

    {{
      "label": "Arduino: Generate Compilation Database",
      "command": "{}; COMPILE_DB_PATH=$(grep '\"path\"' .zed/settings.json 2>/dev/null | grep -B 2 'compileDb' | grep '\"path\"' | grep -o '\"[^\"]*\"' | tail -1 | tr -d '\"'); if [ -n \"$COMPILE_DB_PATH\" ]; then mkdir -p \"$(dirname \"$COMPILE_DB_PATH\")\"; arduino-cli compile --fqbn \"$FQBN\" --build-path \"$(dirname \"$COMPILE_DB_PATH\")\" --only-compilation-database . && mv \"$(dirname \"$COMPILE_DB_PATH\")/compile_commands.json\" \"$COMPILE_DB_PATH\" 2>/dev/null || true; else arduino-cli compile --fqbn \"$FQBN\" --only-compilation-database .; fi",
      "use_new_terminal": true
    }},
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
    }}

    // === Advanced/Diagnostic Tasks ===
    // (Uncomment these if needed for troubleshooting or advanced configuration)

    // {{
    //   "label": "Arduino: Extension Diagnostics",
    //   "command": "echo '=== Extension Installation Status ===' && echo '' && if [ -f installation_state.json ]; then cat installation_state.json | grep -v '{{' | grep -v '}}' || cat installation_state.json; else echo 'No installation state found. Tools may be using system installations.'; fi && echo '' && echo '' && echo '=== Arduino Extension Tool Detection ===' && echo '' && echo 'Checking for clangd...' && (command -v clangd >/dev/null 2>&1 && echo \"  Found in PATH: $(command -v clangd)\" && clangd --version 2>&1 | head -1 || echo '  Not found in PATH') && ([ -f ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd ] && echo \"  Found Zed Flatpak: $(ls ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd 2>/dev/null | head -1)\" && $(ls ~/.var/app/dev.zed.Zed/data/zed/languages/clangd/*/bin/clangd 2>/dev/null | head -1) --version 2>&1 | head -1 || echo '  Zed Flatpak: not found') && echo '' && echo 'Checking for arduino-cli...' && (command -v arduino-cli >/dev/null 2>&1 && echo \"  Found in PATH: $(command -v arduino-cli)\" && arduino-cli version 2>&1 | head -1 || echo '  Not found in PATH') && ([ -f ~/.arduino15/arduino-cli ] && echo '  Found in ~/.arduino15/' || echo '  ~/.arduino15/: not found') && echo '' && echo 'Checking for arduino-cli.yaml config...' && ([ -f ~/.arduino15/arduino-cli.yaml ] && echo '  Found: ~/.arduino15/arduino-cli.yaml' || echo '  Not found in ~/.arduino15/') && ([ -f .arduino-cli.yaml ] && echo '  Found: ./.arduino-cli.yaml' || echo '  Not found in project root') && echo '' && echo 'Environment variables:' && ([ -n \"$CLANGD_PATH\" ] && echo \"  CLANGD_PATH=$CLANGD_PATH\" || echo '  CLANGD_PATH: not set') && ([ -n \"$ARDUINO_CLI_PATH\" ] && echo \"  ARDUINO_CLI_PATH=$ARDUINO_CLI_PATH\" || echo '  ARDUINO_CLI_PATH: not set') && ([ -n \"$ARDUINO_CLI_CONFIG\" ] && echo \"  ARDUINO_CLI_CONFIG=$ARDUINO_CLI_CONFIG\" || echo '  ARDUINO_CLI_CONFIG: not set') && ([ -n \"$ARDUINO_DIRECTORIES_DATA\" ] && echo \"  ARDUINO_DIRECTORIES_DATA=$ARDUINO_DIRECTORIES_DATA\" || echo '  ARDUINO_DIRECTORIES_DATA: not set') && ([ -n \"$ARDUINO_DIRECTORIES_USER\" ] && echo \"  ARDUINO_DIRECTORIES_USER=$ARDUINO_DIRECTORIES_USER\" || echo '  ARDUINO_DIRECTORIES_USER: not set') && echo '' && echo 'Note: The extension checks these locations in priority order:' && echo '  1. Explicit settings (binary.arguments or settings.*)' && echo '  2. Environment variables' && echo '  3. PATH' && echo '  4. Tool-specific locations (shown above)' && echo '  5. Download if not found'",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clear clangd Cache",
    //   "command": "{}",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clear arduino-cli Cache",
    //   "command": "{}",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Show Board Options",
    //   "command": "{}"; echo '' && echo 'Current FQBN:' && echo \"  $FQBN\" && echo '' && arduino-cli board details -b \"$FQBN\" && echo '' && echo '=== How to Use FQBN Options ===' && echo 'Base format: vendor:architecture:board' && echo 'With options: vendor:architecture:board:option1=value1,option2=value2' && echo '' && echo 'Example for your board:' && BASE_FQBN=$(echo \"$FQBN\" | cut -d: -f1-3) && echo \"  $BASE_FQBN:UploadSpeed=921600,FlashFreq=80\" && echo '' && echo 'Copy option values from the rightmost column above (e.g., UploadSpeed=921600)' && echo 'Combine multiple options with commas and append to base FQBN'",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Show Library Dependencies",
    //   "command": "echo 'Enter library name:' && read LIBRARY && arduino-cli lib deps \"$LIBRARY\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: List Examples",
    //   "command": "arduino-cli lib examples",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Regenerate Tasks File",
    //   "command": "echo 'Regenerating .zed/tasks.json...' && rm -f .zed/tasks.json && echo 'Deleted old tasks.json. Restart Zed or run any task to trigger auto-generation.' && echo 'Note: You may need to reload the project (Cmd+Shift+P -> \"zed: reload project\") for changes to take effect.'",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clean Build",
    //   "command": "rm -rf build compile_commands.json *.elf *.hex *.bin && echo 'Build artifacts cleaned'",
    //   "use_new_terminal": false
    // }},
    // {{
    //   "label": "Arduino: Format Code",
    //   "command": "if command -v clang-format >/dev/null 2>&1; then find . -maxdepth 1 -name '*.ino' -exec clang-format -i {{}} \\; && echo 'Code formatted'; else echo 'Error: clang-format not found. Install it with: sudo apt-get install clang-format (Debian/Ubuntu) or brew install clang-format (macOS)'; exit 1; fi",
    //   "use_new_terminal": true
    // }}
  ]
}}
"#,
        readme_path,
        fqbn_extract_helper,
        compile_args_helper,
        upload_args_helper,
        fqbn_extract_helper,
        compile_args_helper,
        upload_args_helper,
        fqbn_extract_helper,
        upload_args_helper,
        fqbn_extract_helper,
        compile_args_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        fqbn_extract_helper,
        r#"echo 'Clearing clangd cache...' && rm -rf .cache/clangd/ ~/.cache/clangd/ && echo 'clangd cache cleared'"#,
        r#"echo 'Clearing arduino-cli cache...' && rm -rf ~/.cache/arduino-cli/ ~/Library/Caches/arduino-cli/ && echo 'arduino-cli cache cleared'"#
    )
}

fn generate_windows_tasks(readme_path: &str) -> String {
    format!(
        r#"{{
  // Arduino Extension Tasks (Windows)
  // For documentation and customization options, see the extension README:
  // {}
  // Or online: https://github.com/SB-CMR-Talana/zed-arduino
  //
  // To regenerate this file:
  // 1. Run the "Arduino: Regenerate Tasks File" task, or
  // 2. Delete this file and restart Zed (auto-generates), or
  // 3. Delete this file and run any Arduino task (triggers auto-generation)
  "tasks": [
    // === Essential Workflow ===
    // (Most frequently used tasks)

    {{
      "label": "Arduino: Compile, Upload & Monitor",
      "command": "powershell -NoProfile -Command \"if (-not (Test-Path .zed)) {{ New-Item -ItemType Directory -Path .zed | Out-Null }}; $settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; $port = $settings.lsp.arduino.settings.port; if ($port -eq 'REPLACE_WITH_YOUR_PORT' -or -not $port) {{ $boardList = arduino-cli board list --format json | ConvertFrom-Json; if ($boardList.Count -gt 0) {{ $port = $boardList[0].port.address }} }}; if (-not $port) {{ Write-Error 'Port not configured and auto-detection failed'; exit 1 }}; $baud = $settings.lsp.arduino.settings.baudRate; if (-not $baud) {{ $baud = 9600 }}; $compileArgs = @(); if ($settings.lsp.arduino.cli.compileArguments) {{ $compileArgs = $settings.lsp.arduino.cli.compileArguments }}; $uploadArgs = @(); if ($settings.lsp.arduino.cli.uploadArguments) {{ $uploadArgs = $settings.lsp.arduino.cli.uploadArguments }}; $compileCmd = @('compile', '-b', $fqbn) + $compileArgs + @('.'); & arduino-cli $compileCmd 2>&1 | Tee-Object -FilePath .zed\\last_compile.log; if ($LASTEXITCODE -eq 0) {{ $uploadCmd = @('upload', '-p', $port, '-b', $fqbn) + $uploadArgs + @('.'); & arduino-cli $uploadCmd; if ($LASTEXITCODE -eq 0) {{ arduino-cli monitor -p $port --config $baud }} }}\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile & Upload",
      "command": "powershell -NoProfile -Command \"if (-not (Test-Path .zed)) {{ New-Item -ItemType Directory -Path .zed | Out-Null }}; $settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; $port = $settings.lsp.arduino.settings.port; if ($port -eq 'REPLACE_WITH_YOUR_PORT' -or -not $port) {{ $boardList = arduino-cli board list --format json | ConvertFrom-Json; if ($boardList.Count -gt 0) {{ $port = $boardList[0].port.address }} }}; if (-not $port) {{ Write-Error 'Port not configured and auto-detection failed'; exit 1 }}; $compileArgs = @(); if ($settings.lsp.arduino.cli.compileArguments) {{ $compileArgs = $settings.lsp.arduino.cli.compileArguments }}; $uploadArgs = @(); if ($settings.lsp.arduino.cli.uploadArguments) {{ $uploadArgs = $settings.lsp.arduino.cli.uploadArguments }}; $compileCmd = @('compile', '-b', $fqbn) + $compileArgs + @('.'); & arduino-cli $compileCmd 2>&1 | Tee-Object -FilePath .zed\\last_compile.log; if ($LASTEXITCODE -eq 0) {{ $uploadCmd = @('upload', '-p', $port, '-b', $fqbn) + $uploadArgs + @('.'); & arduino-cli $uploadCmd }}\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upload (last compile)",
      "command": "powershell -NoProfile -Command \"$settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; $port = $settings.lsp.arduino.settings.port; if ($port -eq 'REPLACE_WITH_YOUR_PORT' -or -not $port) {{ $boardList = arduino-cli board list --format json | ConvertFrom-Json; if ($boardList.Count -gt 0) {{ $port = $boardList[0].port.address }} }}; if (-not $port) {{ Write-Error 'Port not configured and auto-detection failed'; exit 1 }}; $uploadArgs = @(); if ($settings.lsp.arduino.cli.uploadArguments) {{ $uploadArgs = $settings.lsp.arduino.cli.uploadArguments }}; $uploadCmd = @('upload', '-p', $port, '-b', $fqbn) + $uploadArgs + @('.'); & arduino-cli $uploadCmd\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Compile",
      "command": "powershell -NoProfile -Command \"if (-not (Test-Path .zed)) {{ New-Item -ItemType Directory -Path .zed | Out-Null }}; $settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; $compileArgs = @(); if ($settings.lsp.arduino.cli.compileArguments) {{ $compileArgs = $settings.lsp.arduino.cli.compileArguments }}; $compileCmd = @('compile', '-b', $fqbn) + $compileArgs + @('.'); & arduino-cli $compileCmd 2>&1 | Tee-Object -FilePath .zed\\last_compile.log\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Monitor Serial",
      "command": "powershell -NoProfile -Command \"$settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $port = $settings.lsp.arduino.settings.port; if ($port -eq 'REPLACE_WITH_YOUR_PORT' -or -not $port) {{ $boardList = arduino-cli board list --format json | ConvertFrom-Json; if ($boardList.Count -gt 0) {{ $port = $boardList[0].port.address }} }}; if (-not $port) {{ Write-Error 'Port not configured and auto-detection failed'; exit 1 }}; $baud = $settings.lsp.arduino.settings.baudRate; if (-not $baud) {{ $baud = 9600 }}; arduino-cli monitor -p $port --config $baud\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Boards & Ports",
      "command": "arduino-cli board list",
      "use_new_terminal": true
    }},

    // === Board & Hardware Setup ===

    {{
      "label": "Arduino: Search Boards",
      "command": "powershell -NoProfile -Command \"$search = Read-Host 'Enter search term'; arduino-cli board listall | Select-String -Pattern $search -CaseSensitive:$false\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Board Details",
      "command": "powershell -NoProfile -Command \"$settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; arduino-cli board details -b $fqbn\"",
      "use_new_terminal": true
    }},

    // === Core Management ===
    // (Arduino board core installation and updates)

    {{
      "label": "Arduino: Update Core Index",
      "command": "arduino-cli core update-index",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Installed Cores",
      "command": "arduino-cli core list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Install Core",
      "command": "powershell -NoProfile -Command \"arduino-cli core list; Write-Host ''; $core = Read-Host 'Enter core to install (e.g., arduino:avr)'; arduino-cli core install $core\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Uninstall Core",
      "command": "powershell -NoProfile -Command \"arduino-cli core list; Write-Host ''; $core = Read-Host 'Enter core to uninstall (e.g., arduino:avr)'; arduino-cli core uninstall $core\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upgrade All Cores",
      "command": "arduino-cli core upgrade",
      "use_new_terminal": true
    }},

    // === Library Management ===
    // (Arduino library installation and updates)

    {{
      "label": "Arduino: Search Libraries",
      "command": "powershell -NoProfile -Command \"$search = Read-Host 'Enter search term'; arduino-cli lib search $search\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: List Installed Libraries",
      "command": "arduino-cli lib list",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Install Library",
      "command": "powershell -NoProfile -Command \"arduino-cli lib list; Write-Host ''; $library = Read-Host 'Enter library name to install'; arduino-cli lib install $library\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Uninstall Library",
      "command": "powershell -NoProfile -Command \"arduino-cli lib list; Write-Host ''; $library = Read-Host 'Enter library name to uninstall'; arduino-cli lib uninstall $library\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Upgrade All Libraries",
      "command": "arduino-cli lib upgrade",
      "use_new_terminal": true
    }},

    // === Project Management ===

    {{
      "label": "Arduino: Generate Compilation Database",
      "command": "powershell -NoProfile -Command \"$settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; $compileDbPath = $settings.lsp.arduino.compileDb.path; if ($compileDbPath) {{ $buildDir = Split-Path -Parent $compileDbPath; if (-not (Test-Path $buildDir)) {{ New-Item -ItemType Directory -Path $buildDir -Force | Out-Null }}; arduino-cli compile --fqbn $fqbn --build-path $buildDir --only-compilation-database .; if (Test-Path \"$buildDir\\compile_commands.json\") {{ Move-Item -Path \"$buildDir\\compile_commands.json\" -Destination $compileDbPath -Force }} }} else {{ arduino-cli compile --fqbn $fqbn --only-compilation-database . }}\"",
      "use_new_terminal": true
    }},
    {{
      "label": "Arduino: Generate Settings File",
      "command": "powershell -NoProfile -Command \"if (-not (Test-Path .zed)) {{ New-Item -ItemType Directory -Path .zed }}; arduino-cli board list; Write-Host ''; Write-Host 'Creating settings template in .zed\\settings.json'; @'
{{
  \"lsp\": {{
    \"arduino\": {{
      \"settings\": {{
        \"fqbn\": \"arduino:avr:uno\",
        \"port\": \"COM3\",
        \"baudRate\": 9600,
        \"autoCreateConfig\": true
      }}
    }}
  }}
}}
'@ | Out-File -FilePath .zed\\settings.json -Encoding utf8; Write-Host ''; Write-Host 'Created .zed\\settings.json - Edit the FQBN and port above'; Get-Content .zed\\settings.json\"",
      "use_new_terminal": true
    }}

    // === Advanced/Diagnostic Tasks ===
    // (Uncomment these if needed for troubleshooting or advanced configuration)

    // {{
    //   "label": "Arduino: Extension Diagnostics",
    //   "command": "powershell -NoProfile -Command \"Write-Host '=== Extension Installation Status ==='; Write-Host ''; if (Test-Path installation_state.json) {{ Get-Content installation_state.json }} else {{ Write-Host 'No installation state found. Tools may be using system installations.' }}; Write-Host ''; Write-Host ''; Write-Host '=== Arduino Extension Tool Detection ==='; Write-Host ''; Write-Host 'Checking for clangd...'; if (Get-Command clangd -ErrorAction SilentlyContinue) {{ Write-Host \"  Found in PATH: $(Get-Command clangd | Select-Object -ExpandProperty Source)\"; & clangd --version | Select-Object -First 1 }} else {{ Write-Host '  Not found in PATH' }}; Write-Host ''; Write-Host 'Checking for arduino-cli...'; if (Get-Command arduino-cli -ErrorAction SilentlyContinue) {{ Write-Host \"  Found in PATH: $(Get-Command arduino-cli | Select-Object -ExpandProperty Source)\"; & arduino-cli version | Select-Object -First 1 }} else {{ Write-Host '  Not found in PATH' }}; if (Test-Path \"$env:LOCALAPPDATA\\Arduino15\\arduino-cli.exe\") {{ Write-Host \"  Found in $env:LOCALAPPDATA\\Arduino15/\" }} else {{ Write-Host \"  $env:LOCALAPPDATA\\Arduino15/: not found\" }}; Write-Host ''; Write-Host 'Checking for arduino-cli.yaml config...'; if (Test-Path \"$env:LOCALAPPDATA\\Arduino15\\arduino-cli.yaml\") {{ Write-Host \"  Found: $env:LOCALAPPDATA\\Arduino15\\arduino-cli.yaml\" }} else {{ Write-Host \"  Not found in $env:LOCALAPPDATA\\Arduino15/\" }}; if (Test-Path .arduino-cli.yaml) {{ Write-Host '  Found: ./.arduino-cli.yaml' }} else {{ Write-Host '  Not found in project root' }}; Write-Host ''; Write-Host 'Environment variables:'; if ($env:CLANGD_PATH) {{ Write-Host \"  CLANGD_PATH=$env:CLANGD_PATH\" }} else {{ Write-Host '  CLANGD_PATH: not set' }}; if ($env:ARDUINO_CLI_PATH) {{ Write-Host \"  ARDUINO_CLI_PATH=$env:ARDUINO_CLI_PATH\" }} else {{ Write-Host '  ARDUINO_CLI_PATH: not set' }}; if ($env:ARDUINO_CLI_CONFIG) {{ Write-Host \"  ARDUINO_CLI_CONFIG=$env:ARDUINO_CLI_CONFIG\" }} else {{ Write-Host '  ARDUINO_CLI_CONFIG: not set' }}; if ($env:ARDUINO_DIRECTORIES_DATA) {{ Write-Host \"  ARDUINO_DIRECTORIES_DATA=$env:ARDUINO_DIRECTORIES_DATA\" }} else {{ Write-Host '  ARDUINO_DIRECTORIES_DATA: not set' }}; if ($env:ARDUINO_DIRECTORIES_USER) {{ Write-Host \"  ARDUINO_DIRECTORIES_USER=$env:ARDUINO_DIRECTORIES_USER\" }} else {{ Write-Host '  ARDUINO_DIRECTORIES_USER: not set' }}; Write-Host ''; Write-Host 'Note: The extension checks these locations in priority order:'; Write-Host '  1. Explicit settings (binary.arguments or settings.*)'; Write-Host '  2. Environment variables'; Write-Host '  3. PATH'; Write-Host '  4. Tool-specific locations (shown above)'; Write-Host '  5. Download if not found'\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clear clangd Cache",
    //   "command": "powershell -NoProfile -Command \"Write-Host 'Clearing clangd cache...'; Remove-Item -Recurse -Force -ErrorAction SilentlyContinue .cache\\clangd,\"$env:LOCALAPPDATA\\clangd\\cache\"; Write-Host 'clangd cache cleared'\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clear arduino-cli Cache",
    //   "command": "powershell -NoProfile -Command \"Write-Host 'Clearing arduino-cli cache...'; Remove-Item -Recurse -Force -ErrorAction SilentlyContinue \"$env:LOCALAPPDATA\\arduino-cli\\cache\"; Write-Host 'arduino-cli cache cleared'\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Show Board Options",
    //   "command": "powershell -NoProfile -Command \"$settings = Get-Content .zed\\settings.json -Raw | ConvertFrom-Json; $fqbn = $settings.lsp.arduino.settings.fqbn; if (-not $fqbn) {{ Write-Error 'FQBN not found in .zed/settings.json'; exit 1 }}; Write-Host ''; Write-Host 'Current FQBN:'; Write-Host \"  $fqbn\"; Write-Host ''; arduino-cli board details -b $fqbn; Write-Host ''; Write-Host '=== How to Use FQBN Options ==='; Write-Host 'Base format: vendor:architecture:board'; Write-Host 'With options: vendor:architecture:board:option1=value1,option2=value2'; Write-Host ''; Write-Host 'Example for your board:'; $baseFqbn = ($fqbn -split ':')[0..2] -join ':'; Write-Host \"  $baseFqbn:UploadSpeed=921600,FlashFreq=80\"; Write-Host ''; Write-Host 'Copy option values from the rightmost column above (e.g., UploadSpeed=921600)'; Write-Host 'Combine multiple options with commas and append to base FQBN'\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Show Library Dependencies",
    //   "command": "powershell -NoProfile -Command \"$library = Read-Host 'Enter library name'; arduino-cli lib deps $library\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: List Examples",
    //   "command": "arduino-cli lib examples",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Regenerate Tasks File",
    //   "command": "powershell -NoProfile -Command \"Write-Host 'Regenerating .zed\\tasks.json...'; Remove-Item -Force -ErrorAction SilentlyContinue .zed\\tasks.json; Write-Host 'Deleted old tasks.json. Restart Zed or run any task to trigger auto-generation.'; Write-Host 'Note: You may need to reload the project (Cmd+Shift+P -> \"zed: reload project\") for changes to take effect.'\"",
    //   "use_new_terminal": true
    // }},
    // {{
    //   "label": "Arduino: Clean Build",
    //   "command": "powershell -NoProfile -Command \"Remove-Item -Recurse -Force -ErrorAction SilentlyContinue build,compile_commands.json,*.elf,*.hex,*.bin; Write-Host 'Build artifacts cleaned'\"",
    //   "use_new_terminal": false
    // }},
    // {{
    //   "label": "Arduino: Format Code",
    //   "command": "powershell -NoProfile -Command \"if (Get-Command clang-format -ErrorAction SilentlyContinue) {{ Get-ChildItem -Path . -Filter *.ino -File | ForEach-Object {{ clang-format -i $_.FullName }}; Write-Host 'Code formatted' }} else {{ Write-Error 'clang-format not found. Install it from: https://llvm.org/builds/ or via package manager'; exit 1 }}\"",
    //   "use_new_terminal": true
    // }}
  ]
}}
"#,
        readme_path
    )
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
