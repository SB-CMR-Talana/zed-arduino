# Arduino Language Server for Zed

Full Arduino development support in Zed with IntelliSense, diagnostics, and syntax highlighting for `.ino` files.

## Features

- 🎨 Syntax highlighting for Arduino sketches
- 🧠 Code completion, hover info, and go-to-definition
- 🔍 Real-time diagnostics and error checking
- 🔧 Auto-downloads Arduino Language Server and arduino-cli
- ⚡ Zero-config setup - auto-generates project settings

## Quick Start

### 1. Install Extension

In Zed, open the command palette (`Cmd+Shift+P` / `Ctrl+Shift+P`):
- "zed: extensions" → Install Dev Extension → Select this directory

### 2. Open Your Arduino Project

```bash
cd your-arduino-project
zed .
```

### 3. Configure Your Board

The extension will automatically create `.zed/settings.json` in your project. Open it and replace the placeholder with your board's FQBN:

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

Edit the FQBN in the language server arguments:

```json
{
  "lsp": {
    "arduino": {
      "binary": {
        "arguments": [
          "-fqbn",
          "esp32:esp32:esp32s3"
        ]
      }
    }
  }
}
```

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
- **Arduino: Compile** - Verify your sketch compiles
- **Arduino: Upload** - Upload sketch to your board
- **Arduino: Compile & Upload** - Compile then upload in one step
- **Arduino: Monitor Serial** - Open serial monitor
- **Arduino: Clean Build** - Remove build artifacts

### Configure Tasks

Edit `.zed/tasks.json` to set your board and port:

```json
{
  "env": {
    "ZED_ARDUINO_FQBN": "esp32:esp32:esp32s3",
    "ZED_ARDUINO_PORT": "/dev/ttyUSB0"
  }
}
```

**Finding your port:**
```bash
arduino-cli board list                 # Shows connected boards and ports
ls /dev/tty* | grep -i usb            # Linux/macOS
```

**Common ports:**
- Linux: `/dev/ttyUSB0`, `/dev/ttyACM0`
- macOS: `/dev/cu.usbserial-*`, `/dev/cu.usbmodem-*`
- Windows: `COM3`, `COM4`, etc.

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

## Configuration Options

All settings are optional and can be added to `.zed/settings.json`:

| Setting | Default | Description |
|---------|---------|-------------|
| `autoGenerateProjectSettings` | `true` | Auto-create `.zed/settings.json` template |
| `githubRepo` | `arduino/arduino-language-server` | Custom GitHub repo (format: `owner/repo`) |
| `autoDownloadCli` | `true` | Auto-download arduino-cli from GitHub |
| `autoCreateConfig` | `false` | Auto-create `arduino-cli.yaml` if missing |
| `autoInstallCore` | `false` | Auto-install board core for your FQBN |
| `autoGenerateCompileDb` | `false` | Auto-generate `compile_commands.json` |

**Alternative:** Use `binary.path` in LSP settings to specify an absolute path to a manually downloaded language server, bypassing automatic downloads.

### Disable Auto-Generation

If you prefer to manage settings manually:

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

```bash
arduino-cli lib search "Library Name"
arduino-cli lib install "Library Name"
```

Then regenerate compilation database (see above).

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

**Custom language server not downloading?**
- Verify the GitHub repo exists and has releases with Arduino Language Server assets
- Ensure you're using the format `"owner/repo"`, not a full URL
- For non-GitHub providers, use `binary.path` instead
- Check Zed logs: `Cmd+Shift+P` → "zed: open log"

**Check logs:**
In Zed: `Cmd+Shift+P` → "zed: open log"

## Manual Installation (arduino-cli)

The extension auto-downloads arduino-cli by default. To install manually instead:

```bash
# macOS
brew install arduino-cli

# Linux
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh

# Then disable auto-download
# Set "autoDownloadCli": false in settings
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

- [Arduino Language Server](https://github.com/arduino/arduino-language-server)
- [Arduino CLI](https://github.com/arduino/arduino-cli)