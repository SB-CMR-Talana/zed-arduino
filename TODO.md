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

### ✅ Slash Commands for AI Assistant Integration (May 2025)

**Status**: Implemented, tested, and documented - 7 commands total

**Commands Implemented:**

*Context Commands (3):*
- `/arduino-board` - Show current board config (FQBN, port, detected info)
- `/arduino-config` - Dump effective configuration (settings, paths, detected tools)
- `/arduino-sketch` - Show project structure (.ino files, libraries)

*Discovery Commands (4):*
- `/arduino-cores` - List installed board cores and versions
- `/arduino-libraries` - List installed libraries + library search paths (custom & system directories)
- `/arduino-errors` - Show last compilation errors (debugging aid)
- `/arduino-examples` - List available example sketches from libraries

**Features:**
- Markdown output with collapsible sections
- Emoji status indicators (✓, ⚠️)
- No arguments required (simple invocation)
- Leverages existing utility functions
- Clean module separation in `src/slash_commands.rs`
- JSON parsing for arduino-cli commands
- Graceful error handling (e.g., no cores/libraries installed)
- Library search paths displayed (helps AI understand where to find code)

**Documentation:**
- Added to README.md (features list + dedicated section with all 7 commands)
- Registered in extension.toml

**Technical Details:**
- Uses `zed_extension_api` v0.3.0 slash command trait methods
- Commands declared in `extension.toml`
- Returns `SlashCommandOutput` with sectioned text
- Full documentation in `SLASH_COMMANDS.md`

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

### ✅ Task Reorganization (May 2025)

**Reduced from 30 to 27 total tasks (20 active + 7 commented):**

**Removed redundant tasks:**
- ❌ "Show Sketch Size" (redundant - compile output already shows size)
- ❌ "Compile (Verbose)" (users can add `-v` flag manually if needed)

**Merged diagnostic tasks:**
- Combined "Show Extension Status" + "Show Detected Tools" → "Extension Diagnostics"
- New unified diagnostic command shows both installation state and tool detection

**Commented out by default (7 tasks total):**
- Extension Diagnostics (troubleshooting)
- Clear clangd Cache (troubleshooting)
- Clear arduino-cli Cache (troubleshooting)
- Show Board Options (very advanced FQBN customization)
- Show Library Dependencies (debugging only)
- List Examples (can browse online/in IDE)
- Regenerate Tasks File (development only)

**Reorganized by usage frequency:**
1. **Essential Workflow** (5) - Compile & Upload now first (primary workflow)
2. **Board & Hardware Setup** (2)
3. **Core Management** (5) - Grouped together
4. **Library Management** (5) - Grouped together  
5. **Project Management** (3)

**Benefits:**
- Cleaner, more focused task list (20 vs 30)
- Proper workflow ordering (most-used tasks first)
- Logical grouping (cores separate from libraries)
- Advanced/diagnostic tasks don't clutter the list
- Clear section comments for navigation
- Users can easily uncomment optional tasks if needed

**Documentation:**
- Updated README.md task count: "20 Arduino tasks (+7 optional)"
- Updated extension.toml description
- Updated TODO.md with detailed reorganization notes
