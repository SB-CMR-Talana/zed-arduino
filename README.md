# Arduino Language Server for Zed

Full Arduino development support in Zed with IntelliSense, diagnostics, and syntax highlighting for `.ino` and `.pde` files.

## Features

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

## Configuration Reference

All settings go in `.zed/settings.json` under `lsp.arduino.settings`:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| port | string | "" | Serial port (e.g., /dev/ttyUSB0, COM3) |
| libraryPaths | string[] | [] | Custom library directories (absolute or relative) |
| githubRepo | string | "arduino/arduino-language-server" | Custom GitHub repo (owner/repo) |
| languageServerVersion | string | "" | Pin language server version (e.g., "0.7.5") |
| arduinoCliVersion | string | "" | Pin arduino-cli version (e.g., "1.0.4") |
| clangdVersion | string | "" | Pin clangd version (e.g., "18.1.3") |
| autoGenerateProjectSettings | boolean | true | Auto-create .zed/settings.json template |
| autoDownloadCli | boolean | true | Auto-download arduino-cli if not in PATH |
| autoCreateConfig | boolean | false | Auto-create arduino-cli.yaml if missing |
| autoInstallCore | boolean | false | Auto-install board core from FQBN |
| autoGenerateCompileDb | boolean | false | Auto-generate compile_commands.json |

### Minimal Configuration

```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      },
      "settings": {
        "port": "/dev/ttyUSB0"
      }
    }
  }
}
```

### Full Automation

```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      },
      "settings": {
        "autoInstallCore": true,
        "autoGenerateCompileDb": true
      }
    }
  }
}
```

### Version Pinning

```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": ["-fqbn", "esp32:esp32:esp32s3"]
      },
      "settings": {
        "languageServerVersion": "0.7.5",
        "arduinoCliVersion": "1.0.4",
        "clangdVersion": "18.1.3"
      }
    }
  }
}
```

### Custom Language Server

**GitHub fork:**
```jsonc
{
  "lsp": {
    "arduino": {
      "settings": {
        "githubRepo": "yourusername/your-fork"
      }
    }
  }
}
```

**Manual path:**
```jsonc
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

### Custom Library Paths

```jsonc
{
  "lsp": {
    "arduino": {
      "settings": {
        "libraryPaths": [
          "/absolute/path/to/libs",
          "./project-local-libs",
          "../shared-libs"
        ]
      }
    }
  }
}
```

**Note:** Project settings override global settings completely. To combine global + project libraries, list all paths in project settings.

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

**Project Management:**
- **Arduino: Generate Compilation Database** - Create `compile_commands.json` for full IntelliSense
- **Arduino: Clean Build** - Remove all build artifacts and temporary files
- **Arduino: Show Extension Status** - Display installation state (which tools downloaded, where stored)
- **Arduino: Clear clangd Cache** - Fix stale IntelliSense by clearing symbol index
- **Arduino: Clear arduino-cli Cache** - Resolve download/installation issues

**Board & Core Management:**
- **Arduino: Update Core Index** - Refresh list of available board packages
- **Arduino: Search Boards** - Find FQBN for your board (interactive search)
- **Arduino: List Installed Cores** - Show all installed board cores
- **Arduino: Install Core** - Install board package (e.g., esp32:esp32)
- **Arduino: Uninstall Core** - Remove board package
- **Arduino: Upgrade All Cores** - Update all cores to latest versions
- **Arduino: Board Details** - Show specifications for configured board

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

**Linux:**
```bash
# Use your package manager (apt, dnf, pacman, zypper, etc.)
sudo apt install arduino-cli clangd
```

**Windows:**
Download from:
- https://github.com/arduino/arduino-cli/releases
- https://github.com/clangd/clangd/releases

Configure paths in `.zed/settings.json`:

```jsonc
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

## Troubleshooting

**No IntelliSense:**
```bash
arduino-cli compile --fqbn YOUR:BOARD:FQBN --only-compilation-database .
# Restart Zed
```

**Stale completions/errors:**
Run task: **Arduino: Clear clangd Cache**, then restart Zed

**Package installation issues:**
Run task: **Arduino: Clear arduino-cli Cache**, then retry

## Complete Uninstall

1. Uninstall extension in Zed
2. Close Zed completely
3. Remove caches (optional):
   
   **Linux:**
   ```bash
   rm -r ~/.cache/arduino-cli/ ~/.cache/clangd/ .cache/clangd/
   ```
   
   **macOS:**
   ```bash
   rm -r ~/Library/Caches/arduino-cli/ ~/.cache/clangd/ .cache/clangd/
   ```
   
   **Windows (PowerShell):**
   ```powershell
   Remove-Item -Recurse "$env:LOCALAPPDATA\arduino-cli\cache"
   Remove-Item -Recurse "$env:LOCALAPPDATA\clangd\cache"
   Remove-Item -Recurse ".cache\clangd"
   ```

4. Remove system Arduino data (if using system arduino-cli):
   
   **Linux/macOS:**
   ```bash
   rm -r ~/.arduino15/
   ```
   
   **Windows (PowerShell):**
   ```powershell
   Remove-Item -Recurse "$env:USERPROFILE\.arduino15"
   ```

## License

MIT - see [LICENSE](LICENSE)

## Credits

- Original [zed-arduino extension](https://github.com/itzderock/zed-arduino) by Derock Xie
- [Arduino Language Server](https://github.com/arduino/arduino-language-server)
- [Arduino CLI](https://github.com/arduino/arduino-cli)