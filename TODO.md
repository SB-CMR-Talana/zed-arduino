# TODO

## Current Tasks

### Documentation

- **Update README.md**
  - Document new hierarchical settings structure (cli, clangd, ls, compileDb sections)
  - Update example configurations to use nested settings
  - Add migration guide from old flat settings to new nested structure
  - Update settings reference section
  - Ensure examples match SETTINGS.md

## Future Enhancements

### Slash Commands for AI Assistant Integration

**Status**: Deferred - limited utility until AI can auto-invoke commands

**Potential Commands**:
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

**To Revisit When**:
- Zed adds AI agent tool-use capabilities, OR
- When frequently debugging Arduino issues with AI assistant

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

---

## Recent Completions

### ✅ Code Structure Refactoring (May 2025)

Reorganized from functional modules to dependency-based modules:

**Module Organization:**
- `arduino_cli.rs` - Complete Arduino CLI lifecycle
- `clangd.rs` - Complete clangd lifecycle
- `language_server.rs` - Complete language server lifecycle
- `tools.rs` - Tool infrastructure (ToolInfo, versioning, downloads)
- `setup.rs` - Project setup + validation (merged from validation.rs)
- `utils.rs` - Extension utilities (settings, args, env)
- `sketches.rs` - Sketch detection and compile database checking
- `metadata.rs` - Installation state tracking

**Standard API Pattern:**
```rust
const MIN_VERSION / BINARY_NAME / DEFAULT_REPO
pub fn find(worktree) -> Option<ToolInfo>
pub fn get_or_download(worktree, cached_path) -> Result<String>
pub fn validate(path) -> Result<String, String>
pub fn extract_version(path) -> Option<String>
// + tool-specific functions
```

### ✅ Settings Structure Refactoring (April 2025)

**New hierarchical structure:**
```jsonc
"lsp": {
  "arduino": {
    "cli": { "path", "config", "compileArguments", "uploadArguments", ... },
    "clangd": { "path", "arguments", "version" },
    "ls": { "path", "arguments", "version", "githubRepo" },
    "compileDb": { "path" },
    "settings": { "fqbn", "sketchPath", "libraryPaths", "auto*", ... }
  }
}
```

**Migration:**
- Consolidated: `buildPath`, `warnings`, `verbose` → `cli.compileArguments` array
- Moved: `port`, `baudRate` → under `cli` section
- Added: Nested settings support with dot notation
- Created: `SETTINGS.md` comprehensive documentation

### ✅ Code Cleanup

- Removed unused functions and imports
- Consolidated duplicate code
- Consistent logging format ("Arduino: " prefix)
- Clean build: 0 errors, 3 benign warnings (unused standardized API functions)
- Expanded Credits section with all core dependencies (clangd/LLVM, tree-sitter, Zed API)

### ✅ Board Support Documentation (May 2025)

- Added "Supported Boards" section to README showing board-agnostic nature
- Created comprehensive table of popular platforms (AVR, SAMD, ESP32, ESP8266, RP2040, STM32)
- Included FQBNs and board manager URLs for each platform
- Expanded `additionalUrls` examples with all popular board URLs
- Clarified that extension supports ANY arduino-cli compatible board
