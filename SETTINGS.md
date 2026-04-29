# Arduino Extension Settings Structure

## Complete Settings Schema

```jsonc
{
  "lsp": {
    "arduino": {
      // Native Zed settings (takes precedence over nested settings)
      "binary": {
        "path": "",           // Language server binary path (precedence over ls.path)
        "arguments": [],      // Language server arguments (precedence over ls.arguments)
        "env": {}            // Environment variables
      },
      
      // Arduino CLI configuration
      "cli": {
        "path": "",                  // Path to arduino-cli binary
        "config": "",                // Path to arduino-cli.yaml config file
        "compileArguments": [],      // Additional arguments for compile commands
                                     // Example: ["--build-path=build", "--warnings=all", "-v"]
        "uploadArguments": [],       // Additional arguments for upload commands
                                     // Example: ["--programmer=usbtiny", "--verbose"]
        "port": "",                  // Serial port for upload/monitor
        "baudRate": 9600,           // Baud rate for serial monitor
        "version": ""               // Pin to specific arduino-cli version
      },
      
      // Clangd configuration
      "clangd": {
        "path": "",                  // Path to clangd binary
        "arguments": [],             // Additional clangd arguments
                                     // Example: ["--background-index", "--limit-results=100"]
        "version": ""               // Pin to specific clangd version
      },
      
      // Language Server configuration
      "ls": {
        "path": "",                  // Fallback language server path (if binary.path not set)
        "arguments": [],             // Fallback language server args (if binary.arguments not set)
        "version": "",              // Pin to specific language server version
        "githubRepo": ""            // Custom GitHub repo (format: "owner/repo")
                                     // Default: "arduino/arduino-language-server"
      },
      
      // Compilation database configuration
      "compileDb": {
        "path": ""                   // Custom path for compile_commands.json
      },
      
      // Arduino project settings
      "settings": {
        "fqbn": "",                  // Fully Qualified Board Name
                                     // Example: "arduino:avr:uno"
        "sketchPath": "",            // Explicit sketch directory path override
        "libraryPaths": [],          // Custom Arduino library paths
        "additionalUrls": [],        // Board manager additional URLs
                                     // Example: ["https://arduino.esp8266.com/stable/package_esp8266com_index.json"]
        
        // Automation toggles
        "autoCreateConfig": true,         // Auto-create arduino-cli.yaml if missing
        "autoInstallCore": true,          // Auto-install board cores when FQBN detected
        "autoGenerateCompileDb": true,    // Auto-generate compile_commands.json
        "autoDownloadCli": true,          // Auto-download tools if not found
        "autoGenerateTasks": true         // Auto-generate .zed/tasks.json
      }
    }
  }
}
```

## Migration from Previous Settings

| Old Setting | New Setting | Notes |
|-------------|-------------|-------|
| `arduinoCliPath` | `cli.path` | Direct rename |
| `arduinoCliConfig` | `cli.config` | Direct rename |
| `arduinoCliVersion` | `cli.version` | Direct rename |
| `clangdPath` | `clangd.path` | Direct rename |
| `clangdVersion` | `clangd.version` | Direct rename |
| `languageServerVersion` | `ls.version` | Direct rename |
| `githubRepo` | `ls.githubRepo` | Direct rename |
| `port` | `cli.port` | Moved under CLI (CLI-specific) |
| `baudRate` | `cli.baudRate` | Moved under CLI (CLI-specific) |
| `buildPath` | `cli.compileArguments: ["--build-path=..."]` | Use argument array |
| `warnings` | `cli.compileArguments: ["--warnings=all"]` | Use argument array |
| `verbose` | `cli.compileArguments: ["-v"]` | Use argument array |

## Minimal Example

```jsonc
{
  "lsp": {
    "arduino": {
      "settings": {
        "fqbn": "arduino:avr:uno",
        "additionalUrls": [
          "https://arduino.esp8266.com/stable/package_esp8266com_index.json"
        ]
      },
      "cli": {
        "port": "/dev/ttyUSB0",
        "compileArguments": ["--warnings=all", "-v"]
      }
    }
  }
}
```

## Advanced Example

```jsonc
{
  "lsp": {
    "arduino": {
      // Override tool paths
      "cli": {
        "path": "/usr/local/bin/arduino-cli",
        "config": "/home/user/.arduino15/arduino-cli.yaml",
        "port": "/dev/ttyACM0",
        "baudRate": 115200,
        "compileArguments": [
          "--build-path=build",
          "--warnings=all",
          "-v",
          "--jobs=4"
        ],
        "uploadArguments": [
          "--verbose"
        ]
      },
      "clangd": {
        "path": "/usr/bin/clangd-16",
        "arguments": [
          "--background-index",
          "--clang-tidy",
          "--completion-style=detailed",
          "--header-insertion=never"
        ]
      },
      "ls": {
        "version": "0.7.5"
      },
      "settings": {
        "fqbn": "esp8266:esp8266:nodemcuv2",
        "libraryPaths": [
          "/home/user/Arduino/libraries",
          "/home/user/custom_libs"
        ],
        "additionalUrls": [
          "https://arduino.esp8266.com/stable/package_esp8266com_index.json",
          "https://raw.githubusercontent.com/espressif/arduino-esp32/gh-pages/package_esp32_index.json"
        ],
        "autoCreateConfig": false,
        "autoInstallCore": false
      }
    }
  }
}
```

## Setting Precedence

1. **Language Server Binary**: `binary.path` > `ls.path` > auto-detect/download
2. **Language Server Arguments**: `binary.arguments` > `ls.arguments` > defaults
3. **Tool Paths**: Explicit settings > auto-detect > auto-download
4. **All Arguments**: Command-line args (in `binary.arguments`) take precedence over nested setting arrays

## Notes

- All paths can be absolute or relative to workspace
- Empty strings disable that specific setting
- Array settings append to defaults (they don't replace)
- Version pinning prevents automatic updates
- Custom `githubRepo` only applies to language server (not CLI or clangd)
