<div align="center">

  ![File Browser TUI](https://img.shields.io/badge/File-Browser_TUI-blue?style=for-the-badge)
  ![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=for-the-badge&logo=rust)
  ![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
  ![Platform](https://img.shields.io/badge/Platform-Windows-blue?style=for-the-badge)

  # 🗂️ File Browser TUI

  ### A fast, keyboard-driven terminal file browser for Windows

  [Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing)

  ![Screenshot](https://img.shields.io/badge/demo-interfaces/purple?style=for-the-badge)

</div>

---

## Table of Contents

- [Overview](#-overview)
- [Features](#-features)
- [Screenshots](#-screenshots)
- [Quick Start](#-quick-start)
- [Installation](#-installation)
- [Keyboard Shortcuts](#-keyboard-shortcuts)
- [Configuration](#-configuration)
- [Docker Support](#-docker-support)
- [Development](#-development)
- [Testing](#-testing)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

## Overview

**File Browser TUI** is a terminal-based file manager for Windows, inspired by classic tools like `lf`, `ranger`, and `norton Commander`, but built specifically for Windows with Rust and the `ratatui` framework.

It provides a fast, keyboard-driven interface for file navigation and operations, perfect for power users who prefer terminal workflows.

## Features

### 🎯 Core Features

- **Keyboard-Driven Navigation** - Full control without touching the mouse
- **Tabbed Browsing** - Multiple directories open simultaneously
- **Split Dual-Pane View** - Norton Commander-style side-by-side panes
- **Undo/Redo** - Reversible file operations with backup system
- **Content Search** - Grep-like text search inside files
- **Bulk Rename** - Pattern-based batch renaming with preview
- **Multi-Selection** - Select multiple files for batch operations
- **Preview Pane** - Quick preview of text files
- **Bookmarks** - Save and quick-jump to favorite locations

### 🖥️ Windows Optimization

- **Drive Letters** - Easy access to all drives (C:, D:, etc.)
- **UNC Paths** - Support for network paths (`\\server\share`)
- **Windows Attributes** - Proper handling of hidden, system, readonly files
- **Default App Integration** - Open files with Windows default applications

### 🔍 Search & Filter

- **Filename Search** - Incremental search with instant filtering
- **Content Search** - Search for text inside files (async, non-blocking)
- **Extension Filter** - Filter by file type
- **Regex Support** - Advanced pattern matching

### 📁 File Operations

- **Copy/Move** - With progress indication
- **Delete** - With confirmation and undo support
- **Rename** - Single file and batch renaming
- **Create Directory** - With parent directory creation
- **Symlinks** - Create symbolic links and junctions

## Screenshots

```
┌─────────────────────────────────────────────────────────────────┐
│ [1:Projects]  [2:Downloads]  [3:Documents]                    │
├─────────────────────────────────────────────────────────────────┤
│ Path: C:\Users\user\Projects\filebrowser-tui                  │
├─────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────────────────────────────────────────┐ │
│ │  📁 .git/              📁 src/                            │ │
│ │  📁 target/           📄 Cargo.toml                       │ │
│ │  📄 README.md         📄 LICENSE                          │ │
│ │  📄 docker-compose.yml 📄 Dockerfile                        │ │
│ │                                                        │ │
│ │  ┌─────────────────────────────────────────────────┐    │ │
│ │  │ │                                               │    │ │
│ │  │ Preview pane showing file contents...          │    │ │
│ │  │                                               │    │ │
│ │  │ fn main() {                                   │    │ │
│ │  │     println!("Hello, World!");                 │    │ │
│ │  │ }                                            │    │ │
│ │  │                                               │    │ │
│ │  └─────────────────────────────────────────────────┘    │ │
│ └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ NORMAL | 1/8 selected | ^Z:Undo(2) | /:Search | q:Quit         │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Install and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/filebrowser-tui.git
cd filebrowser-tui

# Build and run
cargo build --release
./target/release/fbt

# Or run directly
cargo run
```

### Your First Navigation

```
# Start browsing current directory
./target/release/fbt

# Common keys:
#   j/k or ↓/↑    - Move down/up
#   Enter          - Enter directory / Open file
#   Backspace      - Go to parent directory
#   q              - Quit
```

## Installation

### Requirements

- **Rust** 1.70 or later ([install via rustup](https://rustup.rs/))
- **Windows** 10 or later (for Windows-specific features)
- **Terminal** that supports true color and UTF-8

### Install from Crates.io (when published)

```bash
cargo install filebrowser-tui
```

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/filebrowser-tui.git
cd filebrowser-tui

# Build release binary
cargo build --release

# Binary location
./target/release/fbt
```

### Install System-Wide

```bash
# Copy to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/fbt ~/.local/bin/

# Add to PATH if not already there
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### Windows Installation

```powershell
# Using cargo
cargo install --path .

# Or copy to user profile
mkdir $env:USERPROFILE\bin
Copy-Item target\release\fbt.exe $env:USERPROFILE\bin\
# Then add %USERPROFILE%\bin to your PATH
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `h` / `←` | Go to parent directory / Previous pane (split view) |
| `l` / `→` / `Enter` | Enter directory / Open file / Next pane (split view) |
| `Backspace` | Go to parent directory |
| `Home` | First file |
| `End` | Last file |
| `Page Up/Down` | Scroll page |

### Selection

| Key | Action |
|-----|--------|
| `Space` | Toggle selection |
| `Ctrl+A` | Select all |
| `Ctrl+D` | Deselect all |
| `Ctrl+I` | Invert selection |

### File Operations

| Key | Action |
|-----|--------|
| `Ctrl+C` | Copy selected |
| `Ctrl+X` | Cut selected |
| `Ctrl+V` | Paste |
| `F5` | Copy to other pane (split view) |
| `F6` | Move to other pane (split view) |
| `F7` | Create directory |
| `F8` / `Delete` | Delete (with confirmation) |
| `F2` | Rename |
| `Ctrl+R` | Bulk rename |

### Tabs

| Key | Action |
|-----|--------|
| `Ctrl+T` | New tab (duplicates current directory) |
| `Ctrl+W` | Close current tab |
| `Ctrl+1` to `Ctrl+9` | Switch to tab 1-9 |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |

### View Modes

| Key | Action |
|-----|--------|
| `Ctrl+P` | Toggle split dual-pane view |
| `Tab` | Switch active pane (split view) |
| `p` | Toggle preview pane |
| `.` | Toggle hidden files |
| `Ctrl+L` | Refresh |

### Search

| Key | Action |
|-----|--------|
| `/` or `Ctrl+F` | Filename search |
| `Ctrl+G` | Content search (grep) |
| `n` | Next match |
| `N` | Previous match |

### Sorting

| Key | Action |
|-----|--------|
| `N` | Sort by name |
| `S` | Sort by size |
| `D` | Sort by modified date |
| `T` | Sort by type |

### Undo/Redo

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo last operation |
| `Ctrl+Y` | Redo undone operation |

### Bookmarks

| Key | Action |
|-----|--------|
| `m` | Add bookmark |
| `` ` `` | List bookmarks |
| `0-9` | Quick jump to bookmark |

### Drives (Windows)

| Key | Action |
|-----|--------|
| `Alt+[Letter]` | Switch to drive (e.g., `Alt+D` for D:) |
| `F9` | Show all drives |

### Other

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `~` | Go to home directory |

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%\filebrowser-tui\config.json`
- **Linux**: `~/.config/filebrowser-tui/config.json`

### Default Configuration

```json
{
  "show_hidden": false,
  "sort_by": "Name",
  "sort_order": "Ascending",
  "show_preview": false,
  "preview_width_percent": 40,
  "theme_color": "blue"
}
```

### Bookmarks

Bookmarks are stored in:
- **Windows**: `%APPDATA%\filebrowser-tui\bookmarks.json`
- **Linux**: `~/.config/filebrowser-tui/bookmarks.json`

```json
{
  "bookmarks": [
    {
      "name": "Projects",
      "path": "C:\\Users\\user\\Projects",
      "created_at": "2025-01-15T10:30:00Z"
    }
  ],
  "quick_slots": {
    "1": "C:\\Users\\user\\Documents"
  }
}
```

## Docker Support

### Quick Start with Docker

```bash
# Build and run (mounts current directory)
docker build -t filebrowser-tui:latest .
docker run -it --rm -v "$PWD:/data:rw" filebrowser-tui:latest
```

### Browse Home Directory

```bash
docker run -it --rm \
  -v "$HOME:/data:rw" \
  filebrowser-tui:latest
```

### Using Docker Compose

```bash
docker-compose build
docker-compose run --rm filebrowser
```

### Using the Run Scripts

**Linux/Mac:**
```bash
chmod +x run-docker.sh
./run-docker.sh --path /path/to/browse
```

**Windows PowerShell:**
```powershell
.\run-docker.ps1 -MountPath "C:\Users\user\Documents"
```

See [DOCKER.md](DOCKER.md) for more details.

## Development

### Prerequisites

- Rust 1.70+
- Git
- Optional: Docker for containerized development

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run in development mode
cargo run
```

### Development Commands

```bash
# Check code
cargo check

# Run tests
make test

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Project Structure

```
filebrowser-tui/
├── Cargo.toml           # Dependencies and project config
├── LICENSE              # MIT License
├── README.md            # This file
├── Makefile             # Convenient build commands
├── docker-compose.yml   # Docker orchestration
├── Dockerfile           # Container image
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Application event loop
│   ├── commands.rs      # Keyboard command handling
│   ├── file_ops/        # File operations
│   │   ├── bulk_rename.rs    # Batch renaming
│   │   ├── history.rs         # Undo/redo system
│   │   ├── navigator.rs       # Directory navigation
│   │   ├── operations.rs      # Copy/move/delete/rename
│   │   ├── search.rs          # Content search
│   │   └── windows.rs         # Windows utilities
│   ├── state/           # Application state
│   │   ├── app_state.rs       # Main state machine
│   │   ├── bookmarks.rs       # Bookmark management
│   │   ├── files.rs           # File entry & sorting
│   │   ├── pane.rs            # Split pane state
│   │   ├── selection.rs       # Multi-selection
│   │   └── tab.rs             # Tab management
│   └── ui/              # User interface
│       ├── layout.rs          # Layout calculations
│       ├── render.rs          # Render coordination
│       └── widgets.rs         # UI components
└── tests/               # Integration & E2E tests
```

### Adding Features

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make your changes
3. Write tests for new functionality
4. Ensure all tests pass: `make test`
5. Format code: `cargo fmt`
6. Run linter: `cargo clippy -- -D warnings`
7. Commit your changes
8. Push and create a Pull Request

## Testing

We have comprehensive test coverage:

```bash
# Run all tests
make test

# Unit tests only
make test-unit

# Integration tests
make test-integration

# Property-based tests
make test-property

# Snapshot tests
make test-snapshot

# E2E tests (require built binary)
make test-e2e

# Verbose output
RUST_BACKTRACE=1 cargo test -- --nocapture
```

See [TEST_FIXES.md](TEST_FIXES.md) for details.

## Roadmap

### Completed ✅

- [x] Basic file navigation
- [x] File operations (copy, move, delete, rename, mkdir)
- [x] Multi-selection
- [x] Search and filter
- [x] Bookmarks
- [x] Hidden file toggle
- [x] Sorting options
- [x] Tabbed browsing
- [x] Split dual-pane view
- [x] Undo/redo system
- [x] Content search (grep)
- [x] Bulk rename with patterns
- [x] Preview pane
- [x] Windows drive support
- [x] Docker containerization
- [x] Comprehensive test suite

### Planned 🚧

- [ ] File compression/decompression
- [ ] File properties panel
- [ ] Symbolic link creation in UI
- [ ] Custom themes and color schemes
- [ ] Custom keybindings
- [ ] File association configuration
- [ ] Network share browser
- [ ] Cloud storage integration (OneDrive, Dropbox)
- [ ] Archive browsing (zip, tar)
- [ ] File duplicates finder
- [ ] Disk usage analyzer
- [ ] Built-in file editor
- [ ] Command palette
- [ ] Macros/scripting support

### Future 💡

- [ ] Linux and macOS ports
- [ ] Plugin system
- [ ] Fuzzy matching for search
- [ ] Thumbnail preview for images
- [ ] Audio metadata display
- [ ] Git integration (status, diff view)
- [ ] Remote file system support (SFTP)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### How to Contribute

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust naming conventions
- Write tests for new features
- Keep functions focused and small
- Document public APIs
- Update documentation as needed

### Code Review Process

- All PRs require review before merging
- CI checks must pass
- At least one maintainer approval required

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **ratatui** - Excellent TUI framework for Rust
- **crossterm** - Cross-platform terminal handling
- **ranger** - Inspiration for keybindings
- **lf** - Inspiration for file operations design
- **Norton Commander** - Inspiration for split view
- All contributors and issue reporters

## Support

- 📧 Email: [your-email@example.com]
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/filebrowser-tui/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/filebrowser-tui/discussions)
- 📖 Documentation: [Wiki](https://github.com/yourusername/filebrowser-tui/wiki)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=yourusername/filebrowser-tui&type=Date)](https://star-history.com/#yourusername/filebrowser-tui&Date)

---

<div align="center">

  **Built with ❤️ using Rust and [ratatui](https://github.com/ratatui-org/ratatui)**

  **⭐ Star us on GitHub — it helps!**

</div>
