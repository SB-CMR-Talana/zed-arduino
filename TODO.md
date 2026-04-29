# TODO

## Current Tasks

- Revise readme
- Reexamine all arduino / esp platform support
- Add clangd credit to readme

## Future Enhancements

### Slash Commands for AI Assistant Integration

**Status**: Deferred - limited utility until AI can auto-invoke commands

**Potential Commands to Implement**:
- `/arduino-board` - Show current board config (FQBN, port, detected info)
- `/arduino-config` - Dump effective configuration (settings, paths, detected tools)
- `/arduino-sketch` - Show project structure (.ino files, libraries)
- `/arduino-errors` - Show last compilation errors (if cached)
- `/arduino-serial <duration>` - Capture serial output for X seconds
- `/arduino-upload` - Upload to board and show result

**Current Limitations**:
- Slash commands must be manually invoked by user (AI cannot auto-run them)
- They return static snapshots, not real-time streams
- Most functionality already covered by tasks
- AI can still use terminal commands as workaround

**To Revisit When**:
- Zed adds AI agent tool-use capabilities (letting AI invoke commands), OR
- When frequently debugging Arduino issues with AI assistant

**Benefits if Implemented**:
- Convenient context injection without leaving chat
- Output formatted specifically for AI consumption
- No need to copy-paste from terminal

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

**Implementation Would Require**:
1. Spawn separate `arduino-language-server` for each sketch directory
2. Route LSP requests based on file path to appropriate instance
3. Handle instance lifecycle (start/stop as sketches added/removed)
