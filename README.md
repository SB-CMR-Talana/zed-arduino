# Arduino Language Server for Zed

Full Arduino development support in Zed with IntelliSense, diagnostics, and syntax highlighting for `.ino` files.

## Features

- 🎨 Syntax highlighting for Arduino sketches
- 📝 Code snippets for common Arduino patterns (setup, loop, Serial, etc.)
- 🧠 Code completion, hover info, and go-to-definition
- 🔍 Real-time diagnostics and error checking
- 🔧 Auto-downloads Arduino Language Server and arduino-cli
- ⚡ Zero-config setup - auto-generates project settings
- 🔌 Auto-detects connected boards and configures FQBN and port

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

## Code Snippets

The extension includes 40+ code snippets for common Arduino patterns. Just start typing a prefix and press Tab to expand.

### Main Structure
- `asl` - Complete sketch template with setup() and loop()
- `asetup` - setup() function
- `aloop` - loop() function

### Serial Communication
- `asbeg` - Serial.begin(9600)
- `aspr` - Serial.print()
- `asprl` - Serial.println()
- `asprlbl` - Print with label (e.g., "Value: 123")
- `asav` - Check if Serial.available()
- `asrd` - Serial.read()

### Digital I/O
- `adw` - digitalWrite(pin, HIGH)
- `adr` - digitalRead(pin)
- `apm` - pinMode(pin, OUTPUT)

### Analog I/O
- `aaw` - analogWrite(pin, value)
- `aar` - analogRead(pin)

### Timing
- `adel` - delay(milliseconds)
- `adelus` - delayMicroseconds()
- `amillis` - millis()
- `amicros` - micros()
- `ablink` - Non-blocking blink pattern using millis()

### Control Flow
- `afor` - for loop
- `awhile` - while loop
- `aif` - if statement
- `aifelse` - if-else statement

### Math & Random
- `amap` - map(value, fromLow, fromHigh, toLow, toHigh)
- `aconst` - constrain(value, min, max)
- `arand` - random(min, max)
- `arandseed` - randomSeed(analogRead(0))

### Interrupts
- `aint` - attachInterrupt()
- `adint` - detachInterrupt()
- `aisr` - Interrupt Service Routine function

### Sound
- `atone` - tone(pin, frequency)
- `atoned` - tone(pin, frequency, duration)
- `anotone` - noTone(pin)

### Advanced I/O
- `apulse` - pulseIn(pin, HIGH)
- `ashiftout` - shiftOut()
- `ashiftin` - shiftIn()

### Libraries
- `ainc` - #include <Library.h>
- `adef` - #define constant
- `aconst` - const variable declaration
- `afunc` - function declaration
- `awbeg` - Wire.begin() (I2C)
- `aspibeg` - SPI.begin()
- `aservo` - Servo control
- `alcdbeg` - LCD initialization
- `alcdpr` - LCD print

**Tip:** All snippets start with `a` (for Arduino) to avoid conflicts with other language snippets. Most snippets include helpful placeholders that you can tab through to fill in.

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