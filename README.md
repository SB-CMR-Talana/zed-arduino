# Arduino Language Server for Zed

Full Arduino development support in Zed with IntelliSense, diagnostics, and syntax highlighting for `.ino` and `.pde` files.

## Features

### Comprehensive Tool Detection

The extension provides intelligent, multi-layered tool detection:

#### Detection Priority Order
1. **Explicit Settings**: `binary.arguments` or `lsp.arduino.settings.*`
2. **Environment Variables**: `CLANGD_PATH`, `ARDUINO_CLI_PATH`, `ARDUINO_CLI_CONFIG`
3. **Standard Arduino Environment Variables**: `ARDUINO_DIRECTORIES_DATA`, `ARDUINO_DIRECTORIES_USER`
4. **PATH**: System PATH environment variable
5. **Tool-Specific Locations**: Zed-managed, Arduino IDE installations, package managers
6. **Auto-Download**: Downloads tools if not found anywhere

#### Supported Detection Locations

**clangd:**
- PATH and `CLANGD_PATH` environment variable
- Zed-managed: `~/.var/app/dev.zed.Zed/data/zed/languages/clangd/` (Flatpak)
- Zed-managed: `~/.local/share/zed/languages/clangd/` (standard)
- Zed-managed: `~/Library/Application Support/Zed/languages/clangd/` (macOS)
- System paths: `/usr/bin`, `/usr/local/bin`, `/opt/homebrew/bin`, `/snap/bin`

**arduino-cli:**
- PATH and `ARDUINO_CLI_PATH` environment variable  
- Arduino environment: `$ARDUINO_DIRECTORIES_DATA`, `$ARDUINO_DIRECTORIES_USER`
- System paths: `/usr/bin`, `/usr/local/bin`, `/opt/homebrew/bin`, `/snap/bin`
- Arduino IDE: `~/.arduino15/`, `~/Arduino/`, Flatpak, Snap, macOS, Windows installations

**arduino-cli.yaml:**
- `ARDUINO_CLI_CONFIG` environment variable
- Arduino environment: `$ARDUINO_DIRECTORIES_DATA`, `$ARDUINO_DIRECTORIES_USER`
- Near arduino-cli binary (same directory)
- Project root: `.arduino-cli.yaml`, `arduino-cli.yaml`
- User home: `~/.arduino15/`, `~/.config/arduino-cli/`, `$XDG_CONFIG_HOME`
- Arduino IDE installations

#### Detection Features
- **Version Checking**: Detects and warns about outdated tools (min: clangd 14.0.0, arduino-cli 0.33.0)
- **Symlink Resolution**: Shows both symlink and actual paths
- **Caching**: Caches detection results for faster subsequent starts
- **Verbose Logging**: Shows where each tool was found and its version
- **Diagnostic Task**: Run "Arduino: Show Detected Tools" to see all detection results

###  Core Features

- **Smart Sketch Detection**: Automatically finds Arduino sketches in subdirectories
  - Scans workspace recursively for `.ino`/`.pde` files
  - Selects sketch closest to workspace root (by depth, then alphabetically)
  - Logs detected sketches on startup
  - Works with nested sketch directories
- Auto-downloads Arduino Language Server, arduino-cli, and clangd
- Smart data isolation (extension-managed tools store data in extension dir)
- Auto-detects connected boards (FQBN + port)
- 23 Arduino tasks (compile, upload, library management, etc.)
- 141 code snippets (Arduino core, ESP32, ESP8266, AVR, sensors, networking)
- Custom library path support
- Version pinning for all toolchain components

## Quick Start

1. Install extension in Zed: `Cmd+Shift+P` → "zed: extensions" → Install Dev Extension
2. Open Arduino project: `zed .`
3. Extension auto-generates `.zed/settings.json` with detected board/port
4. Open `.ino` or `.pde` file and start coding

Extension validates dependencies on startup and provides detailed recovery steps for any missing tools.

## Working with Arduino Sketches

### Sketch Detection

The extension automatically scans your workspace for Arduino sketch directories (containing `.ino` or `.pde` files):

- **Single Sketch**: Works seamlessly, logs the detected location
- **Nested Sketches**: Finds sketches in subdirectories (e.g., `projects/my-sketch/`)
- **Multiple Sketches**: Selects the shallowest directory, then alphabetically
  - Logs all detected sketches at startup
  - For multiple independent sketches, **open each as a separate workspace** for best results

### Best Practices

**Single Sketch Project** (Recommended):
```
my-project/
  my-sketch.ino
  config.h
  .zed/
    settings.json
```

**Nested Sketch** (Supported):
```
my-monorepo/
  arduino/
    my-sketch/
      my-sketch.ino  ← Auto-detected
  other-stuff/
```

**Multiple Sketches** (Open separately):
```
my-sketches/
  sketch-a/     ← Open this directory in Zed
    sketch-a.ino
  sketch-b/     ← Open this directory in a new Zed window
    sketch-b.ino
```

> **Note**: Due to Zed extension API limitations, only one language server instance can run per workspace. For multiple independent sketches, open each sketch directory as its own workspace.

## Configuration Reference

All settings go in `.zed/settings.json` under `lsp.arduino.settings`:

| Setting | Default | Description |
|---------|---------|-------------|
| fqbn | | **Required:** Fully Qualified Board Name (e.g., arduino:avr:uno, esp32:esp32:esp32) |
| port | | Serial port (e.g., /dev/ttyUSB0, COM3) |
| baudRate | 9600 | Baud rate for serial monitor (e.g., 9600, 115200) |
| sketchPath | | Override sketch directory path (for multi-sketch workspaces) |
| clangdPath | | Path to clangd binary (auto-detected if not specified) |
| arduinoCliPath | | Path to arduino-cli binary (auto-detected/downloaded if not specified) |
| arduinoCliConfig | | Path to arduino-cli.yaml config file (auto-detected if not specified) |
| libraryPaths | | Custom library directories (absolute or relative) |
| additionalUrls | | Board manager additional URLs (e.g., for ESP32/ESP8266) |
| buildPath | | Custom build directory path (default: system temp) |
| warnings | "none" | Compiler warnings level: "none", "default", "more", "all" |
| verbose | false | Enable verbose compilation output |
| programmer | | Programmer for ISP uploads (e.g., "usbtiny", "usbasp") |
| uploadProtocol | | Override upload protocol (rarely needed) |
| githubRepo | "arduino/arduino-language-server" | GitHub repo (owner/repo) |
| languageServerVersion | | Pin language server version (e.g., "0.7.5") |
| arduinoCliVersion | | Pin arduino-cli version (e.g., "1.0.4") |
| clangdVersion | | Pin clangd version (e.g., "18.1.3") |
| autoGenerateTasks | true | Auto-create .zed/tasks.json with Arduino commands |
| autoDownloadCli | true | Auto-download arduino-cli if not in PATH |
| autoCreateConfig | true | Auto-create arduino-cli.yaml if missing |
| autoInstallCore | true | Auto-install board core from FQBN |
| autoGenerateCompileDb | true | Auto-generate compile_commands.json for better IntelliSense |

### Minimal Configuration

```jsonc
{
  "lsp": {
    "arduino": {
      "settings": {
        "fqbn": "arduino:avr:uno",
        "port": "/dev/ttyUSB0"
      }
    }
  }
}
```

**Legacy format** (still supported for backward compatibility):
```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": ["-fqbn", "arduino:avr:uno"]
      },
      "settings": {
        "port": "/dev/ttyUSB0"
      }
    }
  }
}
```

### Maximal Configuration

```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "path": "/absolute/path/to/arduino-language-server"  // Optional: override auto-download
      },
      "settings": {
        "fqbn": "esp32:esp32:esp32s3",
        "port": "/dev/ttyUSB0",
        "baudRate": 115200,
        "sketchPath": "projects/my-sketch",           // Optional: for multi-sketch workspaces
        "clangdPath": "/path/to/clangd",              // Optional: override auto-detection
        "arduinoCliPath": "/path/to/arduino-cli",     // Optional: override auto-detection
        "arduinoCliConfig": "/path/to/arduino-cli.yaml",  // Optional: override auto-detection
        "libraryPaths": [
          "/path/to/libraries",
          "./relative/path"
        ],
        "additionalUrls": [                           // Optional: for ESP32/ESP8266/etc
          "https://espressif.github.io/arduino-esp32/package_esp32_index.json",
          "https://arduino.esp8266.com/stable/package_esp8266com_index.json"
        ],
        "buildPath": "./build",                       // Optional: persistent build directory
        "warnings": "all",                            // Optional: "none", "default", "more", "all"
        "verbose": true,                               // Optional: verbose compilation
        "programmer": "usbtiny",                      // Optional: for ISP uploads
        "uploadProtocol": "serial",                   // Optional: override protocol
        "githubRepo": "owner/repo",
        "languageServerVersion": "x.y.z",
        "arduinoCliVersion": "x.y.z",
        "clangdVersion": "x.y.z",
        "autoGenerateTasks": true,
        "autoDownloadCli": true,
        "autoCreateConfig": true,
        "autoInstallCore": true,
        "autoGenerateCompileDb": true
      }
    }
  }
}
```

**Legacy binary.arguments format** (still supported):
```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": [
          "-fqbn", "esp32:esp32:esp32s3",
          "-cli", "/path/to/arduino-cli",
          "-clangd", "/path/to/clangd",
          "-cli-config", "/path/to/arduino-cli.yaml"
        ]
      }
    }
  }
}
```

## Available Tasks

Access via `Cmd+Shift+P` → `tasks: spawn`

**Core Workflow:**
- **Arduino: List Boards & Ports** - Detect connected boards with FQBN and serial port
- **Arduino: Compile** - Verify sketch compiles without errors
- **Arduino: Compile (Verbose)** - Compile with detailed compiler output for debugging
- **Arduino: Upload (last compile)** - Upload previously compiled binary (faster, auto-detects port)
- **Arduino: Compile & Upload** - Full workflow: compile then upload to board
- **Arduino: Monitor Serial** - Open serial monitor (auto-detects port if not configured)
- **Arduino: Show Sketch Size** - Display memory usage and compiled size

<br>

**Project Management:**
- **Arduino: Generate Compilation Database** - Create `compile_commands.json` for full IntelliSense
- **Arduino: Clean Build** - Remove all build artifacts and temporary files
- **Arduino: Show Extension Status** - Display installation state (which tools downloaded, where stored)
- **Arduino: Clear clangd Cache** - Fix stale IntelliSense by clearing symbol index
- **Arduino: Clear arduino-cli Cache** - Resolve download/installation issues

<br>

**Board & Core Management:**
- **Arduino: Update Core Index** - Refresh list of available board packages
- **Arduino: Search Boards** - Find FQBN for your board (interactive search)
- **Arduino: List Installed Cores** - Show all installed board cores
- **Arduino: Install Core** - Install board package (e.g., esp32:esp32)
- **Arduino: Uninstall Core** - Remove board package
- **Arduino: Upgrade All Cores** - Update all cores to latest versions
- **Arduino: Board Details** - Show specifications for configured board

<br>

**Library Management:**
- **Arduino: Search Libraries** - Find libraries by name (interactive search)
- **Arduino: List Installed Libraries** - Show all installed libraries
- **Arduino: Install Library** - Add library from Arduino registry
- **Arduino: Uninstall Library** - Remove installed library
- **Arduino: Upgrade All Libraries** - Update all libraries to latest versions
- **Arduino: Show Library Dependencies** - Display library dependency tree
- **Arduino: List Examples** - Browse example sketches from installed libraries

## Code Snippets

141 snippets covering:
- **Core:** `sketch`, `setup`, `loop`, `serial`, `digital`, `analog`, `millis`
- **ESP32:** `esp32wifi`, `esp32ble`, `esp32spiffs`, `esp32webserver`, `esp32task`
- **ESP8266:** `esp8266wifi`, `esp8266webserver`, `esp8266ota`
- **AVR:** `avreeprom`, `avrsleep`, `avrwatchdog`
- **Sensors:** `mpu6050`, `dht`, `bme280`, `ultrasonic`, `gps`
- **Displays:** `oled`, `lcd`, `tft`
- **Motors:** `stepper`, `servo`, `dcmotor`
- **Networking:** `mqtt`, `httpget`, `websocket`, `json`
- **Patterns:** `statemachine`, `debounce`, `pid`, `nonblocking`

Type snippet prefix and use autocomplete.

## Data Storage

**System arduino-cli (in PATH):**
- Uses `~/.arduino15/` (standard Arduino ecosystem)
- Shares cores/libraries with Arduino IDE

**Extension-downloaded arduino-cli:**
- Stores cores/libraries in extension directory (isolated)
- Cache still stored externally (see below)
- Clean uninstall - removes cores/libraries but not cache

**Caches (external to extension):**
- arduino-cli: `~/.cache/arduino-cli/` (Linux), `~/Library/Caches/arduino-cli/` (macOS), `%LOCALAPPDATA%\arduino-cli\cache\` (Windows)
- clangd: `<project>/.cache/clangd/` and `~/.cache/clangd/`
- Both auto-regenerate if deleted

## Manual Installation

To use manually installed tools instead of auto-download:

**macOS:**
```bash
brew install arduino-cli clang
```

<br>

**Linux:**

Debian/Ubuntu:
```bash
sudo apt install arduino-cli clangd
```

Fedora:
```bash
sudo dnf install arduino-cli clang-tools-extra
```

Arch:
```bash
sudo pacman -S arduino-cli clang
```

openSUSE:
```bash
sudo zypper install arduino-cli clang
```

<br>

**Windows:**
Download from:
- https://github.com/arduino/arduino-cli/releases
- https://github.com/clangd/clangd/releases

Add to your PATH or use environment variables.

<br>

**That's it!** The extension will automatically detect installed tools through:
- PATH
- Standard installation locations
- Zed-managed installations
- Environment variables (`CLANGD_PATH`, `ARDUINO_CLI_PATH`, `ARDUINO_CLI_CONFIG`)

No configuration needed unless you want to override detection. See [Configuration Reference](#configuration-reference) for optional settings.

## Troubleshooting

**No IntelliSense:**

```bash
arduino-cli compile --fqbn YOUR:BOARD:FQBN --only-compilation-database .
```

Restart Zed

<br>

**Stale completions/errors:**

Run task: **Arduino: Clear clangd Cache**, then restart Zed

<br>

**Package installation issues:**

Run task: **Arduino: Clear arduino-cli Cache**, then retry


## Complete Uninstall

1. Uninstall extension in Zed

2. (Optional) Remove caches and data:
   
   Use tasks: Open an Arduino project and run **Arduino: Clear clangd Cache** and **Arduino: Clear arduino-cli Cache**
   
   Or manually:
   
   **Linux:**
   ```bash
   rm -r ~/.cache/arduino-cli/ ~/.cache/clangd/ .cache/clangd/
   rm -r ~/.arduino15/  # If using system arduino-cli
   ```
   
   **macOS:**
   ```bash
   rm -r ~/Library/Caches/arduino-cli/ ~/.cache/clangd/ .cache/clangd/
   rm -r ~/.arduino15/  # If using system arduino-cli
   ```
   
   **Windows (PowerShell):**
   ```powershell
   Remove-Item -Recurse "$env:LOCALAPPDATA\arduino-cli\cache"
   Remove-Item -Recurse "$env:LOCALAPPDATA\clangd\cache"
   Remove-Item -Recurse ".cache\clangd"
   Remove-Item -Recurse "$env:USERPROFILE\.arduino15"  # If using system arduino-cli
   ```

3. (Optional) Remove Arduino configuration files:
   
   Project settings: `.zed/settings.json` in each Arduino project
   
   Project tasks: `.zed/tasks.json` in each Arduino project
   
   Global settings: Remove Arduino LSP settings from Zed's global settings file

## License

MIT - see [LICENSE](LICENSE)

## Credits

- Original [zed-arduino extension](https://github.com/itzderock/zed-arduino) by Derock Xie
- [Arduino Language Server](https://github.com/arduino/arduino-language-server)
- [Arduino CLI](https://github.com/arduino/arduino-cli)
