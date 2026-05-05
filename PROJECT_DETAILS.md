# Project Details & Debugging

## Zed Extension Sandboxing

Zed extensions run in a WASM (WebAssembly) environment with restricted capabilities:

### File System Access
- **File I/O (`fs::read_dir`, `fs::write`, etc.)**: ✓ Works, but operates in sandboxed location
- **Shell Commands (`std::process::Command`)**: ✗ Does NOT work - fails with "operation not supported on this platform"
- **Extension API Commands (`zed_extension_api::process::Command`)**: ✅ **WORKS!**
  - This is the proper API for running commands from extensions
  - Unlike `std::process::Command`, this goes through the extension host
  - Works in WASM/flatpak environments
  - **Requires capability declaration in `extension.toml`**
  - **Returns `Vec<u8>` for stdout/stderr** - use `String::from_utf8_lossy()` to convert
  - **`.arg()` takes ownership** - use builder pattern: `cmd = cmd.arg("x").arg("y")`

**Current Implementation**: The extension consistently uses `zed_extension_api::process::Command` throughout:
- `src/sketches.rs` - Sketch detection with `find` command
- `src/arduino_cli.rs` - All arduino-cli operations (version check, board detection, core install, compile)
- `src/clangd.rs` - Clangd version checking
- `src/language_server.rs` - Language server version checking
- `src/slash_commands.rs` - All slash command implementations

**Version Validation**: All tools now validate versions against minimum requirements:
- Arduino CLI: minimum 0.33.0
- Clangd: minimum 14.0.0
- Arduino Language Server: validated on download

### Known Issue: Clangd Indexing Performance

The standard Arduino Language Server forces clangd to use `--pch-storage=memory`, which means:
- Precompiled headers are stored in RAM only
- Full reindex on every LSP restart (5-10 seconds)
- No persistent cache benefits

Zed displays indexing progress at the bottom of the window during startup.

**Workaround**: Build a custom Arduino LS that uses `--pch-storage=disk` instead. The flag is hardcoded in `ls/lsp_client_clangd.go` line 51. Simply change `--pch-storage=memory` to `--pch-storage=disk` for persistent caching and faster restarts.

**Our Solution**: The extension generates `compile_commands.json` smartly (only when missing or FQBN changes) to minimize the 10-30 second compilation database generation time. Clangd indexing cannot be avoided with standard Arduino LS, but it is relatively fast and Zed shows progress.

**This affects**:
- ✅ Automatic sketch detection - **WORKING** using proper API with capabilities
- ❌ General file operations for user's workspace (no generic write API available)

**API Version**: Updated from `0.3.0` to `0.7.0` to test if newer versions fixed sandbox restrictions - they did not. The sandbox behavior is intentional security design.

### Extension Capabilities

Zed uses a capability system to control what operations extensions can perform. To use `zed_extension_api::process::Command`, the extension must declare the required capability in `extension.toml`:

```toml
capabilities = [{ kind = "process:exec", command = "find", args = ["**"] }]
```

This declaration:
- Lists which commands the extension needs to execute
- Is checked when the extension attempts to run a command
- Does **NOT** require users to manually grant permissions in their settings
- Is automatically approved for dev extensions and published extensions reviewed by Zed

**Note**: Users can optionally restrict capabilities via `granted_extension_capabilities` in their settings, but this is not required for normal operation.

## Sketch Detection
These hyper-specific capabilities show users **exactly** what the extension does, addressing security concerns.

### Zed Extension Locations

```bash
# Extension installation directory (symlinked to dev source)
~/.var/app/dev.zed.Zed/data/zed/extensions/installed/arduino -> /home/talana/desktop/projects/Software/custom-als-zed

# Compiled extension WASM file
~/.var/app/dev.zed.Zed/data/zed/extensions/installed/arduino/extension.wasm

# Zed logs
~/.var/app/dev.zed.Zed/data/zed/logs/Zed.log
```

## Language Server Logs

The Arduino language server logs are written to `/tmp/` (as specified by `-logpath /tmp` argument):

```bash
# Find recent language server logs
ls -lt /tmp/arduino-language-server-*.log | head -5
```

## Development Workflow

### Rebuilding the Extension

1. Make changes to source code
2. In Zed, go to Extensions panel
3. Click the "Rebuild" button next to the Arduino extension (only visible for dev-installed extensions)
4. Restart the language server or reload the window

### Checking if Rebuild Worked

```bash
# Check recent builds in Zed log
tail -100 ~/.var/app/dev.zed.Zed/data/zed/logs/Zed.log | grep -E "compiling|compiled|arduino"

# Check WASM file modification time
ls -l ~/.var/app/dev.zed.Zed/data/zed/extensions/installed/arduino/extension.wasm
```

## Sketch Detection

**Important**: Automatic sketch detection is **not possible** in Zed's WASM extension environment due to fundamental sandbox limitations.

### Why Auto-Detection Doesn't Work

1. **`std::process::Command` fails** - Shell commands return "operation not supported on this platform"
2. **`fs::read_dir` is sandboxed** - Only reads the extension's sandbox directory, not the real workspace
3. **Worktree API has no directory listing** - The `Worktree` type only provides: `id()`, `root_path()`, `read_text_file()`, `which()`, and `shell_env()`

### Current Behavior

- **Default**: Uses workspace root (`.`) as sketch path
- **For subdirectory sketches**: Users **must** set `sketchPath` in settings

### Configuring Sketch Path

Add to your `.zed/settings.json` or global settings:

```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "sketchPath": "test_sketch"
      }
    }
  }
}
```

The `sketchPath` should be a relative path from the workspace root to the directory containing your `.ino` file.

### Removed Code

Previous versions attempted automatic sketch detection using:
- Recursive `fs::read_dir` traversal
- Shell commands with `find`

Both approaches failed due to WASM sandbox restrictions. The detection code has been removed in favor of clear documentation that users must configure `sketchPath` for subdirectory sketches.

## Tasks Configuration

**Setup**: Users must manually copy task templates to their workspace.

### Task Templates

The extension provides ready-to-use task templates in the `templates/` directory:
- `templates/unix_tasks.json` - For Linux and macOS
- `templates/windows_tasks.json` - For Windows

Users should:
1. Copy the appropriate template to their workspace as `.zed/tasks.json`
2. Customize the settings (FQBN, port, etc.) in `.zed/settings.json`
3. Optionally uncomment advanced/diagnostic tasks at the bottom of the file

### Why Manual Copy?

Automatic task generation was removed because:
- Zed's extension sandbox prevents reliable file writes to user workspaces
- Manual templates are simpler, more transparent, and easier to customize
- Users have full control over which tasks are available
- Templates can be version-controlled with the project

## Common Issues

### Extension Not Using Latest Code

- Make sure to click "Rebuild" in the Extensions panel
- Check that `extension.wasm` has a recent modification time
- Look for "compiled Rust extension" messages in Zed log

### Sketch Detection Not Working

- Check Zed's log panel for sketch detection messages (`Cmd+Shift+P` → "zed: open log")
- Verify `.ino` file exists and is not in a skipped directory (test/, grammars/, examples/)
- Ensure file has correct extension (`.ino` or `.pde`)
- If sketch is in a subdirectory, set `sketchPath` in settings

### Language Server Failing to Start

- Check `/tmp/arduino-language-server-*.log` for errors
- Verify sketch path is correct in Zed log (look for `-sketch-path` argument)
- Ensure arduino-cli and clangd are available
