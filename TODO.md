# TODO

## Current Tasks

### Documentation

- **CHANGELOG.md** - Track version history and changes for users (needed before first release)
- **TROUBLESHOOTING.md** - Common issues guide:
  - Port access denied (permissions)
  - Board not detected
  - Compilation errors
  - IntelliSense not working
  - Windows-specific issues
  - Platform-specific troubleshooting

### Optional Enhancements

- **Additional Snippet Libraries** (nice-to-have alternatives to existing snippets):
  - U8g2 display library (alternative to Adafruit GFX/SSD1306)
  - TB6612FNG motor driver (alternative to L298N)

## Future Enhancements

### Context-Aware C++/H File Routing

**Status**: Blocked by Zed extension API limitations

**Current Implementation**:
- All `.cpp`, `.c`, `.h`, `.hpp` files are marked as "Arduino" language (`languages/arduino/config.toml`)
- This prevents Zed's built-in clangd from handling them
- All C/C++ files in workspace go to Arduino Language Server

**Trade-offs**:
- ✅ Arduino Language Server handles all C/C++ files (correct Arduino IntelliSense)
- ✅ Works for pure Arduino projects
- ⚠️ Cannot mix Arduino projects with regular C++ projects in same workspace
- ⚠️ All C++ files get Arduino syntax highlighting and treatment

**Ideal Solution** (not currently possible):
- Route only C++ files within sketch directory to Arduino LS
- Route other C++ files to built-in clangd
- Requires Zed API support for:
  - Dynamic language ID assignment based on file location
  - Per-directory language server routing
  - Extension-provided language servers taking precedence over built-in ones for specific paths

**To Revisit When**:
Zed extension API adds support for:
- Path-based language server routing (e.g., "route \*.cpp in sketch_dir/ to Arduino LS")
- Extension language servers with priority/precedence over built-in servers
- Dynamic `path_suffixes` or conditional language assignment

### Automatic Board Detection & FQBN Resolution

**Status**: ✅ Completed

**Implementation**:
- ✓ Auto-detects connected Arduino boards via `arduino-cli board list`
- ✓ 4-tier FQBN priority: user settings → detected board → last detected → error
- ✓ Persists last detected board to `installation_state.json` (never writes to user settings)
- ✓ Board detection includes FQBN, port, and board name
- ✓ Works transparently - logs what FQBN is being used
- ✓ Requires no user configuration if board is connected

**User Experience**:
- Users can plug in Arduino and extension "just works"
- Last detected board remembered across sessions
- User settings always take precedence (no surprise auto-configuration)
- Clear logging: "Auto-detected board: Arduino Uno (FQBN: arduino:avr:uno)"

### Multi-Sketch Language Server Support

**Status**: Partially implemented - requires manual configuration

**Current Implementation**:
- ✓ Sketch path argument (`-sketch-path`) now passed to language server
- ✓ Shell command approach for detection (using `find` command)
- ⚠️ Detection currently not working reliably - returns empty results
- ✓ Users can manually set `sketchPath` in settings as workaround
- ✓ Documentation guides users to open sketches separately

**Known Issues**:
- Shell command detection (`find` with `-exec dirname`) returns empty list in WASM
- Need to debug why `std::process::Command` with `find` doesn't capture output correctly
- Possible issues: output parsing, relative vs absolute paths, or command execution context

**Workaround**:
Users should set `sketchPath` in workspace settings:
```json
{
  "lsp": {
    "arduino": {
      "settings": {
        "sketchPath": "main"
      }
    }
  }
}
```

**To Fix**:
- Debug why `find` command output isn't being captured properly
- Consider alternative approaches:
  - Try `sh -c` with full pipeline instead of `find -exec`
  - Add verbose logging to see command output and errors
  - Test if `Command::output()` works differently in Zed's WASM vs standalone
- Once working, remove workaround note from documentation

**To Revisit When**:
Zed extension API adds support for:
- Multiple LS instances per worktree, OR
- Per-file/per-directory LS routing, OR
- Multi-root workspace concepts (like VS Code)
