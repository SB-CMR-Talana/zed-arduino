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

**Status**: Deferred - blocked by Zed extension API limitations

**Current Implementation**:
- ✓ Detects all sketches in workspace recursively
- ✓ Uses first sketch (by depth, then alphabetically)
- ✓ Logs detected sketches and warns if multiple found
- ✓ Documentation guides users to open sketches separately

**To Revisit When**:
Zed extension API adds support for:
- Multiple LS instances per worktree, OR
- Per-file/per-directory LS routing, OR
- Multi-root workspace concepts (like VS Code)
