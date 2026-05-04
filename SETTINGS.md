# Arduino Extension Settings

Use this structure in your Zed `settings.json` (global) or `.zed/settings.json` (workspace).

```jsonc
{
  // ...other Zed settings...
  "lsp": {
    // ...other language server settings...
    "arduino": {
      // Native Zed LSP binary settings
      "binary": {
        "path": "",          // Optional: explicit arduino-language-server binary
        "arguments": [],      // Optional: full argument list (takes precedence over ls.arguments)
        "env": {}             // Optional: environment variables
      },

      // Extension-specific settings live under "settings"
      "settings": {
        // Arduino CLI
        "cli": {
          "path": "",                // Optional: arduino-cli path
          "config": "",              // Optional: arduino-cli.yaml path
          "compileArguments": [],      // Extra args for compile database generation / compile invocations
          "uploadArguments": [],       // Extra args for upload invocations
          "port": "",                // Optional: serial port (e.g. /dev/ttyUSB0, COM3)
          "baudRate": 9600,            // Optional: serial monitor baud rate
          "version": ""              // Optional: pin arduino-cli version for downloads
        },

        // Clangd
        "clangd": {
          "path": "",                // Optional: clangd path
          "arguments": [],             // Optional: extra clangd args
          "version": ""              // Optional: pin clangd version for downloads
        },

        // Arduino Language Server download/config settings
        "ls": {
          "path": "",                // Optional fallback if binary.path is not set
          "arguments": [],             // Used when binary.arguments is empty
          "version": "",             // Optional: pin arduino-language-server version for downloads
          "githubRepo": ""           // Optional: custom repo in "owner/repo" format
        },

        // Project settings
        "fqbn": "",                  // Fully Qualified Board Name, e.g. "arduino:avr:uno"
        "sketchPath": "",            // Optional explicit sketch directory path
        "libraryPaths": [],            // Optional custom Arduino library paths
        "additionalUrls": [],          // Optional board manager URLs for third-party platforms

        // Compilation database
        "compileDb": {
          "path": ""                 // Optional custom compile_commands.json location
        },

        // Automation
        "autoCreateConfig": true,
        "autoInstallCore": true,
        "autoGenerateCompileDb": true,
        "autoDownloadCli": true,
        "autoGenerateTasks": true
      }
    }
  }
}
```

## Minimal Example

```jsonc
{
  "lsp": {
    "arduino": {
      "settings": {
        "fqbn": "arduino:avr:uno",
        "cli": {
          "port": "/dev/ttyUSB0"
        }
      }
    }
  }
}
```

## Path Setup Example (Global)

```jsonc
{
  "lsp": {
    "arduino": {
      "binary": {
        "path": "/usr/local/bin/arduino-language-server"
      },
      "settings": {
        "cli": {
          "path": "/usr/local/bin/arduino-cli",
          "config": "/home/you/.arduino15/arduino-cli.yaml"
        },
        "clangd": {
          "path": "/usr/bin/clangd"
        },
        "ls": {
          "path": "/usr/local/bin/arduino-language-server"
        }
      }
    }
  }
}
```

## Precedence

1. `lsp.arduino.binary.path` takes precedence over `lsp.arduino.settings.ls.path`.
2. `lsp.arduino.binary.arguments` takes precedence over `lsp.arduino.settings.ls.arguments`.
3. Explicit tool paths are preferred over auto-detection/download.
