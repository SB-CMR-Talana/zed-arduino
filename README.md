# Arduino Language Server for Zed

Full Arduino development support in Zed with IntelliSense, diagnostics, and syntax highlighting for `.ino` files.

## Features

- 🎨 Syntax highlighting for Arduino sketches
- 📝 Code snippets for common Arduino patterns and platform-specific features (ESP32, ESP8266, AVR)
- 🧠 Code completion, hover info, and go-to-definition
- 🔍 Real-time diagnostics and error checking
- 🔧 Auto-downloads Arduino Language Server, arduino-cli, and clangd
- 🗂️ Smart data isolation - arduino-cli downloaded by extension stores cores/libraries in extension directory for clean uninstall
- ⚡ Zero-config setup - auto-generates project settings
- 🔌 Auto-detects connected boards and configures FQBN and port
- 📚 Custom library path support for project-specific libraries

## Quick Start

### 1. Install Extension

In Zed, open the command palette (`Cmd+Shift+P` / `Ctrl+Shift+P`):
- "zed: extensions" → Install Dev Extension → Select this directory

### 2. Open Your Arduino Project

```bash
cd your-arduino-project
zed .
```

### 3. Configure Your Board & Port

The extension will automatically create `.zed/settings.json` in your project with **auto-detected values** if you have a board connected. If no board is detected, placeholders will be used - just edit them:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "autoGenerateProjectSettings": true,
        "languageServerRepo": "arduino/arduino-language-server"
      }
    }
  },
  "languages": {
    "Arduino": {
      "language_servers": ["arduino"],
      "format_on_save": "off"
    }
  }
}
```

Edit the FQBN and port in the settings:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": [
          "-fqbn",
          "esp32:esp32:esp32s3"
        ]
      },
      "settings": {
        "port": "REPLACE_WITH_YOUR_PORT"
      }
    }
  }
}
```

**Note:** If you had a board connected when opening the project, the FQBN and port should already be set correctly! Otherwise, run the **Arduino: List Boards & Ports** task (see [Running Arduino Tasks](#running-arduino-tasks) below) to find your values.

**Multiple Boards:** If multiple boards are connected, the extension will use the first detected board and display a warning with all detected boards. Edit `.zed/settings.json` manually if you need to use a different board.

**That's it!** Open your `.ino` file and start coding.

## Finding Your Board FQBN

```bash
arduino-cli board listall              # List all boards
arduino-cli board list                 # Detect connected board
```

**Common FQBNs:**
- Arduino Uno: `arduino:avr:uno`
- Arduino Mega: `arduino:avr:mega`
- ESP32: `esp32:esp32:esp32`
- ESP32-S3: `esp32:esp32:esp32s3`
- ESP8266: `esp8266:esp8266:generic`

## Running Arduino Tasks

The extension auto-generates `.zed/tasks.json` with common Arduino commands. Access them via:

**Command Palette** → `Cmd+Shift+P` / `Ctrl+Shift+P` → `tasks: spawn`

Available tasks:

**Core Workflow:**
- **Arduino: List Boards & Ports** - Detect connected boards and their ports
- **Arduino: Compile** - Verify your sketch compiles
- **Arduino: Compile (Verbose)** - Compile with detailed output for debugging
- **Arduino: Upload (last compile)** - Upload previously compiled binary (auto-detects port if not configured)
- **Arduino: Compile & Upload** - Compile then upload in one step
- **Arduino: Monitor Serial** - Open serial monitor (auto-detects port if not configured)
- **Arduino: Show Sketch Size** - Display compiled sketch size and memory usage

**Project Management:**
- **Arduino: Generate Compilation Database** - Create/update `compile_commands.json` for IntelliSense
- **Arduino: Clean Build** - Remove all build artifacts (build/, *.elf, *.hex, *.bin, compile_commands.json)
- **Arduino: Show Extension Status** - Display which tools were installed by the extension and their data storage locations
- **Arduino: Clear clangd Cache** - Remove clangd index cache (project and system) to resolve stale IntelliSense issues
- **Arduino: Clear arduino-cli Cache** - Remove arduino-cli download cache to resolve package installation issues

**Board & Core Management:**
- **Arduino: Update Core Index** - Update available board packages
- **Arduino: Search Boards** - Search for board FQBNs (interactive)
- **Arduino: List Installed Cores** - Show installed board cores
- **Arduino: Install Core** - Install a board core (shows installed cores first, then prompts)
- **Arduino: Uninstall Core** - Remove an installed board core (shows installed cores first, then prompts)
- **Arduino: Upgrade All Cores** - Update all installed board cores to latest versions
- **Arduino: Board Details** - Show detailed information about the configured board

**Library Management:**
- **Arduino: Search Libraries** - Search for available libraries (interactive)
- **Arduino: List Installed Libraries** - Show all installed libraries
- **Arduino: Install Library** - Install a library (shows installed libraries first, then prompts)
- **Arduino: Uninstall Library** - Remove an installed library (shows installed libraries first, then prompts)
- **Arduino: Upgrade All Libraries** - Update all installed libraries to latest versions
- **Arduino: Show Library Dependencies** - Display dependencies for a library (interactive)
- **Arduino: List Examples** - Browse available example sketches from installed libraries

### Understanding Upload Tasks

**Arduino: Upload (last compile)** vs **Arduino: Compile & Upload**:

- **Upload (last compile)** - Uploads the existing binary from your last compilation. Faster if you just want to re-upload the same code to another board or re-flash without changes. Fails if you haven't compiled yet.

- **Compile & Upload** - Always compiles fresh before uploading. Use this for your normal workflow when you've made code changes. Guarantees you're uploading the latest version.

**Recommendation:** Use **Compile & Upload** for most development. Only use **Upload (last compile)** when you want to quickly re-upload an unchanged binary.

### Understanding Cache Clearing Tasks

**When to use cache clearing tasks:**

- **Arduino: Clear clangd Cache** - Use when IntelliSense shows stale completions, outdated symbols, or incorrect errors after adding libraries or changing code structure. This clears the C++ symbol index cache.

- **Arduino: Clear arduino-cli Cache** - Use when experiencing package download failures, corrupted core installations, or "hash mismatch" errors during core/library installation.

**Important:** These tasks are for **troubleshooting during normal use**, NOT for uninstallation. If you're uninstalling the extension, close Zed first and manually delete cache directories - otherwise the tools will immediately recreate them.

**What gets cleared:**
- clangd cache: Project `.cache/clangd/` and system cache directory
- arduino-cli cache: System cache directory (temporary download files)

Both caches will be automatically regenerated as needed when you continue working.

### Configure Tasks

**Both FQBN and port are automatically extracted from `.zed/settings.json`** - everything in one place!

**Smart Port Auto-Detection:** If you haven't configured a port yet (or it's set to `REPLACE_WITH_YOUR_PORT`), upload and monitor tasks will automatically detect and use the first connected board's port. This means you can start uploading immediately after connecting a board!

If you need to change your board or port (or if auto-detection didn't find your board), edit `.zed/settings.json`:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      },
      "settings": {
        "port": "REPLACE_WITH_YOUR_PORT"
      }
    }
  }
}
```

**Finding your port:**
1. Run the **Arduino: List Boards & Ports** task (`Cmd+Shift+P` → `tasks: spawn`)
2. Or manually: `arduino-cli board list`

**Common ports:**
- Linux: `/dev/ttyUSB0`, `/dev/ttyACM0`
- macOS: `/dev/cu.usbserial-*`, `/dev/cu.usbmodem-*`
- Windows: `COM3`, `COM4`, etc.

### Task Features

**Error Handling:** All tasks include helpful error messages. If a task fails (e.g., FQBN not configured, port not found), you'll see a clear explanation of what went wrong.

**Interactive Tasks:** Install and uninstall tasks automatically show what's currently installed before prompting you for input. This helps you avoid duplicates and see exactly what's available to remove. Search tasks will prompt you for a search term.

**Verbose Output:** Use **Arduino: Compile (Verbose)** when you need detailed compiler output for debugging build issues.

**Customizing Terminal Output:**  
By default, most tasks open in a terminal panel for better visibility. If you prefer inline output instead, edit `.zed/tasks.json` and change `"use_new_terminal": true` to `false` for any task.

## Using a Custom Language Server

### Option 1: GitHub Fork

To use a forked Arduino Language Server hosted on GitHub:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "githubRepo": "yourusername/your-arduino-language-server"
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

The extension will download releases from the specified GitHub repository (format: `"owner/repo"`).

**Note:** Only GitHub is supported for automatic downloads. For GitLab, Gitea, or other providers, use Option 2 below.

### Option 2: Manual Path

To use a manually downloaded language server from any source:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "path": "/absolute/path/to/arduino-language-server",
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

This skips automatic downloads entirely and uses your specified binary.

### Option 3: Version Pinning

Pin specific versions of the Arduino Language Server, arduino-cli, and/or clangd to ensure consistent behavior across your team and complete toolchain control.

**Pin Language Server version:**

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "languageServerVersion": "0.7.5"
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

**Pin arduino-cli version:**

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "arduinoCliVersion": "1.0.4"
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

**Pin clangd version:**

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "clangdVersion": "18.1.3"
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

**Pin all versions (recommended for maximum reproducibility):**

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "languageServerVersion": "0.7.5",
        "arduinoCliVersion": "1.0.4",
        "clangdVersion": "18.1.3"
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

Versions can be specified with or without the `v` prefix for languageServerVersion and arduinoCliVersion (e.g., `"0.7.5"` or `"v0.7.5"`). For clangd, use the version number directly (e.g., `"18.1.3"`).

**Why pin versions?**
- Ensure consistent behavior across team members
- Avoid breaking changes from new releases
- Test compatibility with specific versions
- Reproducible builds and toolchain management

**Note:** Leave version settings empty or omit them to always use the latest versions. The `languageServerVersion` setting works with both the default `arduino/arduino-language-server` repo and custom GitHub repositories specified via `githubRepo`.

## Full Automation (Optional)

For a completely hands-off experience, enable automatic core installation and compilation database generation:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "autoDownloadCli": true,
        "autoInstallCore": true,
        "autoGenerateCompileDb": true
      },
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      }
    }
  }
}
```

**Note:** `autoInstallCore` and `autoGenerateCompileDb` may download large files (100MB+) and take several minutes on first run.

## Custom Library Paths

If you have project-specific libraries or want to use libraries from custom locations, you can specify them in your settings:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "libraryPaths": [
          "/path/to/custom/libraries",
          "./project-libs",
          "../shared-libraries"
        ]
      }
    }
  }
}
```

**Features:**
- Supports multiple library directories
- Can use absolute or relative paths
- Relative paths are resolved from the project root
- Libraries are automatically included during compilation and IntelliSense

**Global vs Project Settings:**
- Project settings (`.zed/settings.json`) override global settings completely
- If you define `libraryPaths` in both, **only the project paths are used**
- To use both global and project libraries, you must list all paths in the project settings
- Example: If global has `["/global/lib"]` and project has `["./local"]`, only `["./local"]` is used

**Practical Example - Combining Global and Project Libraries:**

Global settings (`~/.config/zed/settings.json`):
```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "libraryPaths": ["/home/user/shared-arduino-libs"]
      }
    }
  }
}
```

Project settings (`.zed/settings.json` in your project):
```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "libraryPaths": [
          "/home/user/shared-arduino-libs",
          "./local-libs"
        ]
      }
    }
  }
}
```

This way, your project uses both the global shared libraries and its local libraries.

**When to use:**
- Custom or modified libraries not in arduino-cli's library manager
- Project-specific libraries
- Shared libraries across multiple projects
- Development versions of libraries

**Note:** After adding library paths, regenerate the compilation database:
- Run task: `Arduino: Generate Compilation Database`
- Or enable `autoGenerateCompileDb: true` in settings

## Configuration Options

All settings are optional and can be added to `.zed/settings.json`:

| Setting | Default | Description |
|---------|---------|-------------|
| `autoGenerateProjectSettings` | `true` | Auto-create `.zed/settings.json` template |
| `githubRepo` | `arduino/arduino-language-server` | Custom GitHub repo (format: `owner/repo`) |
| `languageServerVersion` | `""` (latest) | Pin to specific language server version (e.g., `"0.7.5"` or `"v0.7.5"`) |
| `autoDownloadCli` | `true` | Auto-download arduino-cli from GitHub |
| `arduinoCliVersion` | `""` (latest) | Pin to specific arduino-cli version (e.g., `"1.0.4"` or `"v1.0.4"`) |
| `clangdVersion` | `""` (latest) | Pin to specific clangd version (e.g., `"18.1.3"`) |
| `autoCreateConfig` | `false` | Auto-create `arduino-cli.yaml` if missing |
| `autoInstallCore` | `false` | Auto-install board core for your FQBN |
| `autoGenerateCompileDb` | `false` | Auto-generate `compile_commands.json` |

**Alternative:** Use `binary.path` in LSP settings to specify an absolute path to a manually downloaded language server, bypassing automatic downloads.

### Available Settings

All settings go under `"lsp" → "arduino" → "settings"`:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `autoGenerateProjectSettings` | boolean | `true` | Auto-generate `.zed/settings.json` and `.zed/tasks.json` |
| `autoDownloadCli` | boolean | `true` | Auto-download arduino-cli if not in PATH |
| `autoCreateConfig` | boolean | `false` | Auto-create minimal `arduino-cli.yaml` config |
| `autoInstallCore` | boolean | `false` | Auto-install board core when FQBN is detected |
| `autoGenerateCompileDb` | boolean | `false` | Auto-generate compilation database for IntelliSense |
| `githubRepo` | string | `"arduino/arduino-language-server"` | Custom GitHub repo for language server |
| `languageServerVersion` | string | `""` | Pin specific language server version |
| `arduinoCliVersion` | string | `""` | Pin specific arduino-cli version |
| `clangdVersion` | string | `""` | Pin specific clangd version |
| `port` | string | `""` | Serial port for uploads (e.g., `/dev/ttyUSB0`) |
| `libraryPaths` | array | `[]` | Custom library directories (absolute or relative paths) |

### Disable Auto-Generation

If you want to manually manage `.zed/settings.json`, disable auto-generation:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "autoGenerateProjectSettings": false
      }
    }
  }
}
```

## Data Storage & Isolation

The extension uses **smart data isolation** to ensure clean uninstalls while maintaining compatibility with the Arduino ecosystem.

### **How It Works:**

#### **arduino-cli found in PATH (System Installation)**
If arduino-cli is already installed on your system:
- ✅ Uses your existing `~/.arduino15/` directory
- ✅ Shares board cores and libraries with Arduino IDE, PlatformIO, etc.
- ✅ No duplication of large files (cores can be 200MB+ each)
- ✅ Standard Arduino ecosystem behavior

#### **arduino-cli downloaded by extension**
If the extension downloads arduino-cli for you:
- ✅ Stores ALL data in extension directory (isolated)
- ✅ Board cores stored in extension directory
- ✅ Libraries stored in extension directory
- ✅ Clean uninstall - removing extension removes everything
- ✅ No pollution of your home directory

### **Installation Tracking:**

The extension automatically tracks which tools it downloaded in `installation_state.json`:

```json
{
  "platform": "linux",                // "linux" | "macos" | "windows"
  "arduino_cli": {
    "source": "downloaded",           // "downloaded" = isolated mode
    "version": "1.0.4",
    "location": "/path/to/arduino-cli",
    "uses_isolated_data": true
  }
}
```

### **Why This Matters:**

- **For new Arduino users:** Everything stays isolated - uninstalling the extension removes all Arduino data
- **For existing Arduino users:** Your existing `~/.arduino15/` setup is used - no duplication, full compatibility
- **For CI/CD:** Clean, isolated environments that don't pollute the system

### **Manual Override:**

If you want to force a specific behavior, use explicit paths:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": [
          "-cli", "/usr/local/bin/arduino-cli",  // Use system arduino-cli
          "-fqbn", "esp32:esp32:esp32s3"
        ]
      }
    }
  }
}
```

### **External Cache Files:**

Even with isolated mode, some tools may still create cache files outside the extension directory. Here's the **complete picture**:

#### **1. arduino-cli Cache (Small, Temporary)**

**Location (cannot be configured):**
- Linux: `~/.cache/arduino-cli/`
- macOS: `~/Library/Caches/arduino-cli/`
- Windows: `%LOCALAPPDATA%\arduino-cli\cache\`

**What's cached:**
- HTTP download cache during core/library installation
- Temporary package extraction files
- Network request cache

**Size & Behavior:**
- Typically <50MB
- Automatically cleaned by arduino-cli after successful installations
- Regenerated as needed

**Impact:** Minimal - standard application cache behavior (like browser cache)

---

#### **2. clangd Index Cache (IDE Performance)**

**Location:**
- **Primary:** `<your-project>/.cache/clangd/` (project-specific)
- **Alternative:** `~/.cache/clangd/` or `$XDG_CACHE_HOME/clangd/` (system-wide)

**What's cached:**
- Symbol index for fast code navigation
- Precompiled headers for faster parsing
- AST (Abstract Syntax Tree) cache

**Size & Behavior:**
- Varies by project: 10-100MB per project
- Automatically regenerated if deleted
- Speeds up IntelliSense significantly

**Control:** Standard IDE cache (like `.vscode/` or `.idea/`). Add to `.gitignore`:
```gitignore
.cache/
```

---

#### **3. Arduino Language Server (No External Data)**

The Arduino Language Server itself writes **no cache or data files** outside the extension directory. It operates purely in-memory and via LSP protocol.

---

#### **4. Temporary System Files**

Like any application, these tools may use:
- `/tmp/` or `$TMPDIR` on Linux/macOS
- `%TEMP%` on Windows

These are automatically cleaned by the OS.

---

### **Complete External Writes Summary:**

| Component | Writes Outside Extension/Project | Location | Size | Auto-Cleaned | User Impact |
|-----------|----------------------------------|----------|------|--------------|-------------|
| **Extension binaries** | ❌ No | Extension dir only | N/A | N/A | None |
| **arduino-cli data** (isolated mode) | ❌ No | Extension dir | 0-500MB | On uninstall | None |
| **arduino-cli data** (PATH mode) | ✅ Yes | `~/.arduino15/` | 0-500MB | Manual | Standard Arduino |
| **arduino-cli cache** | ✅ Yes | System cache dir | <50MB | By arduino-cli | Minimal |
| **clangd cache** | ✅ Yes | Project `.cache/` | 10-100MB | Regenerates | IDE performance |
| **Arduino Language Server** | ❌ No | N/A | 0MB | N/A | None |
| **System temp files** | ✅ Yes | `/tmp/`, `%TEMP%` | <10MB | By OS | None |

---

### **Manual Cache Cleanup:**

**Important:** These cache directories are **NOT automatically deleted** by your operating system. If you want to reclaim disk space after uninstalling the extension, you'll need to manually remove them.

#### **Locating Cache Directories:**

**Linux:**
- arduino-cli cache: `~/.cache/arduino-cli/`
- clangd cache: `~/.cache/clangd/`
- Project clangd cache: `<your-project>/.cache/clangd/`

**macOS:**
- arduino-cli cache: `~/Library/Caches/arduino-cli/`
- clangd cache: `~/.cache/clangd/` or `~/Library/Caches/clangd/`
- Project clangd cache: `<your-project>/.cache/clangd/`

**Windows:**
- arduino-cli cache: `%LOCALAPPDATA%\arduino-cli\cache\`
- clangd cache: `%LOCALAPPDATA%\clangd\cache\`
- Project clangd cache: `<your-project>\.cache\clangd\`

#### **Cleanup Commands:**

**Linux/macOS:**
```bash
# Remove arduino-cli cache
rm -rf ~/.cache/arduino-cli/

# Remove clangd system cache
rm -rf ~/.cache/clangd/

# Remove clangd project cache (run in your project directory)
rm -rf .cache/clangd/

# If using system arduino-cli (not isolated mode), remove all Arduino data
rm -rf ~/.arduino15/
```

**Windows (PowerShell):**
```powershell
# Remove arduino-cli cache
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\arduino-cli\cache"

# Remove clangd cache
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\clangd\cache"

# Remove clangd project cache (run in your project directory)
Remove-Item -Recurse -Force ".cache\clangd"

# If using system arduino-cli (not isolated mode), remove all Arduino data
Remove-Item -Recurse -Force "$env:USERPROFILE\.arduino15"
```

**Windows (Command Prompt):**
```cmd
rem Remove arduino-cli cache
rmdir /s /q "%LOCALAPPDATA%\arduino-cli\cache"

rem Remove clangd cache
rmdir /s /q "%LOCALAPPDATA%\clangd\cache"

rem Remove clangd project cache (run in your project directory)
rmdir /s /q ".cache\clangd"

rem If using system arduino-cli (not isolated mode)
rmdir /s /q "%USERPROFILE%\.arduino15"
```

**Note:** These cache directories improve IDE performance. Only delete them if you're reclaiming disk space or troubleshooting issues. They will be recreated automatically when needed.

---

## Complete Uninstallation

To completely remove the extension and all associated data:

### **Step 1: Uninstall the Extension**

In Zed, open the command palette (`Cmd+Shift+P` / `Ctrl+Shift+P`):
- "zed: extensions" → Find "Arduino Language Support" → Uninstall

**What this removes automatically:**
- ✅ Extension code
- ✅ Downloaded binaries (arduino-language-server, arduino-cli, clangd)
- ✅ If arduino-cli was downloaded by extension: All board cores and libraries in extension directory
- ✅ Installation metadata

### **Step 2: Remove Project Configuration (Optional)**

If you want to remove Arduino configuration from your projects:

```bash
# In each Arduino project directory
rm -rf .zed/settings.json .zed/tasks.json
```

### **Step 3: Close Zed**

**Important:** Close Zed completely before cleaning cache directories. If Zed is running with an Arduino project open, clangd and arduino-cli will recreate their caches immediately.

### **Step 4: Remove Cache Directories (Optional)**

**These are NOT removed automatically.** Clean them manually if desired (do NOT use the cache clearing tasks for uninstallation):

**Linux/macOS:**
```bash
rm -rf ~/.cache/arduino-cli/
rm -rf ~/.cache/clangd/
```

**Windows (PowerShell):**
```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\arduino-cli\cache"
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\clangd\cache"
```

**Note:** The cache clearing tasks (`Arduino: Clear clangd Cache` and `Arduino: Clear arduino-cli Cache`) are for troubleshooting during normal use, not for uninstallation. Use the manual commands above after closing Zed.

### **Step 5: Remove System Arduino Installation (If Applicable)**

**Only do this if:**
- You installed arduino-cli system-wide (not via the extension)
- You don't use Arduino IDE or other Arduino tools
- You want to completely remove all Arduino data

**Linux/macOS:**
```bash
rm -rf ~/.arduino15/
# If installed via package manager:
# macOS: brew uninstall arduino-cli
# Linux: sudo apt remove arduino-cli (or equivalent)
```

**Windows (PowerShell):**
```powershell
Remove-Item -Recurse -Force "$env:USERPROFILE\.arduino15"
```

---

### **What Gets Left Behind:**

After uninstalling the extension, the following may remain on your system:

| Item | Location | Size | Auto-Removed |
|------|----------|------|--------------|
| **Extension binaries & data** | Extension work directory | Varies | ✅ Yes |
| **Project .zed/ config** | Each project | <10KB | ❌ No |
| **arduino-cli cache** | System cache directory | <50MB | ❌ No |
| **clangd cache** | Project & system | 10-100MB | ❌ No |
| **System arduino-cli data** | `~/.arduino15/` | 0-500MB | ❌ No (only if you installed it separately) |

**For a 100% clean system:** Follow all 5 steps above.

---

## Installing Board Cores

First time using a board? Install its core:

```bash
arduino-cli core update-index
arduino-cli core install esp32:esp32  # or arduino:avr, etc.
```

Or enable `autoInstallCore: true` in settings.

## Getting Full IntelliSense

For complete code completion with all library headers:

```bash
arduino-cli compile --fqbn esp32:esp32:esp32s3 --only-compilation-database .
```

Or enable `autoGenerateCompileDb: true` in settings.

**After adding libraries:** Regenerate the compilation database and restart Zed.

## Installing Libraries

**Standard libraries (from arduino-cli):**

```bash
arduino-cli lib search "Library Name"
arduino-cli lib install "Library Name"
```

**Custom/project-specific libraries:**

Add them to your project and configure library paths in `.zed/settings.json`:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "libraryPaths": ["./libraries", "/path/to/custom/libs"]
      }
    }
  }
}
```

**After installing or adding libraries:** Regenerate the compilation database:
- Run task: `Arduino: Generate Compilation Database`
- Or enable `autoGenerateCompileDb: true` in settings
- Then restart Zed for full IntelliSense

## Code Snippets

The extension includes 130+ insertable code snippets. Just start typing and use autocomplete to discover them.

### Categories

**Core Arduino:** `sketch`, `setup`, `loop`, `serial`, `digital`, `analog`, `delay`, `millis`, etc.

**Platform-Specific:** Type platform name to discover snippets
- `esp32*` - WiFi, BLE, SPIFFS, deep sleep, web server, tasks
- `esp8266*` - WiFi, web server, OTA, LittleFS, deep sleep
- `avr*` - EEPROM, sleep modes, watchdog, timers
- `rp2040*` - Dual-core, PIO, flash, temperature sensor
- `samd*` - RTC, low power modes
- `teensy*` - Audio, USB device emulation
- `stm32*` - Hardware timers, DMA, low power

**Sensors & Displays:** `oled`, `lcd`, `tft`, `mpu6050`, `dht`, `bme280`, `ultrasonic`, `gps`, etc.

**Motors:** `stepper`, `accelstepper`, `dcmotor`, `servo`

**Networking:** `mqtt`, `httpget`, `httppost`, `websocket`, `json`

**Storage:** `sdread`, `sdwrite`, `csvlog`

**LEDs:** `neopixel`, `fastled` with rainbow effects

**Patterns:** `statemachine`, `debounce`, `pid`, `movingavg`, `ringbuffer`, `nonblocking`, `ema`, `median`

**Communication:** `i2c`, `spi`, `softserial`

All snippets include helpful placeholders you can tab through to customize.

## Troubleshooting

**Settings file not auto-generated?**
- Check that `autoGenerateProjectSettings` is not explicitly set to `false`
- The file won't be created if `.zed/settings.json` already exists

**No code completion?**
```bash
arduino-cli compile --fqbn YOUR:BOARD:FQBN --only-compilation-database .
# Restart Zed
```

**Library not found?**
```bash
arduino-cli lib install "Library Name"
arduino-cli compile --fqbn YOUR:BOARD:FQBN --only-compilation-database .
```

**IntelliSense not updating or stale completions?**
Run the **Arduino: Clear clangd Cache** task (`Cmd+Shift+P` → `tasks: spawn`), then restart Zed.

Alternatively, clear manually:
```bash
# Linux/macOS
rm -rf .cache/clangd/
rm -rf ~/.cache/clangd/

# Windows (PowerShell)
Remove-Item -Recurse -Force ".cache\clangd"
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\clangd\cache"
```

**arduino-cli behaving strangely or download issues?**
Run the **Arduino: Clear arduino-cli Cache** task (`Cmd+Shift+P` → `tasks: spawn`), then retry your operation.

Alternatively, clear manually:
```bash
# Linux/macOS
rm -rf ~/.cache/arduino-cli/

# macOS (alternative location)
rm -rf ~/Library/Caches/arduino-cli/

# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\arduino-cli\cache"
```

**Custom language server not downloading?**
- Verify the GitHub repo exists and has releases with Arduino Language Server assets
- Ensure you're using the format `"owner/repo"`, not a full URL
- For non-GitHub providers, use `binary.path` instead
- Check Zed logs: `Cmd+Shift+P` → "zed: open log"

**Check logs:**
In Zed: `Cmd+Shift+P` → "zed: open log"

## Manual Installation

The extension auto-downloads all required components by default. To install manually instead:

### Arduino CLI

```bash
# macOS
brew install arduino-cli

# Linux
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh

# Windows
# Download from https://github.com/arduino/arduino-cli/releases
```

### clangd

```bash
# macOS
brew install llvm

# Linux (Ubuntu/Debian)
sudo apt install clangd

# Arch Linux
sudo pacman -S clang

# Or let Zed install it by opening any C++ file
```

### Arduino Language Server

Download from [GitHub releases](https://github.com/arduino/arduino-language-server/releases) or use a custom fork via the `githubRepo` setting.

### Disable Auto-Downloads

To use manually installed versions, they must be in your PATH, or you can configure explicit paths in `.zed/settings.json`:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "path": "/path/to/arduino-language-server",
        "arguments": [
          "-cli", "/path/to/arduino-cli",
          "-clangd", "/path/to/clangd",
          "-fqbn", "esp32:esp32:esp32s3"
        ]
      }
    }
  }
}
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

- [Arduino Language Server](https://github.com/arduino/arduino-language-server)
- [Arduino CLI](https://github.com/arduino/arduino-cli)