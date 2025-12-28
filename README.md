# XNote

<div align="center">

![XNote Logo](src-tauri/icons/128x128.png)

**A lightweight, cross-platform Markdown note-taking application**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/HuskarTang/xnote)
[![Release](https://img.shields.io/badge/release-v0.1.0-blue.svg)](https://github.com/HuskarTang/xnote/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](#installation)

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Development](#development) • [Contributing](#contributing)

![XNote Screenshot](attachments/Clipboard_2025-09-05-16-23-00.png)

</div>

---

XNote is a modern, cross-platform note-taking application built with Vue 3, TypeScript, and Tauri. It combines the power of Markdown with an intuitive interface, offering seamless note organization through tags and powerful search capabilities.

## Features

- **Markdown Editing**: Full support for Markdown syntax with live preview
- **Tag-based Organization**: Organize notes with custom tags
- **File Attachments**: Attach files and images to notes
- **Search Functionality**: Powerful search across all notes
- **Favorites**: Mark important notes as favorites
- **Trash/Recycle Bin**: Safely delete notes with recovery option
- **Cross-platform**: Works on Windows and macOS
- **Real-time Saving**: Automatic saving with manual override
- **Split View**: Edit and preview simultaneously

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust (Tauri)
- **Data Storage**: 
  - Markdown files (.md)
  - SQLite database (mdnote.db) for metadata and tags
  - Attachments directory for media files
- **Configuration**: JSON format (config.json)

## 🚀 Installation

### Download Pre-built Releases

**[📥 Download the latest release](https://github.com/HuskarTang/xnote/releases/latest)**

| Platform | Download |
|----------|----------|
| 🪟 **Windows** | [XNote-setup.exe](https://github.com/HuskarTang/xnote/releases/latest/download/XNote-setup.exe) |
| 🍎 **macOS** | [XNote.dmg](https://github.com/HuskarTang/xnote/releases/latest/download/XNote.dmg) |
| 🐧 **Linux** | [XNote.AppImage](https://github.com/HuskarTang/xnote/releases/latest/download/XNote.AppImage) |

### Build from Source

#### Prerequisites

- **Node.js** (v16 or higher) - [Download](https://nodejs.org/)
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **Git** - [Download](https://git-scm.com/)

#### Quick Start

```bash
# Clone the repository
git clone https://github.com/HuskarTang/xnote.git
cd xnote

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

#### Building for Production

```bash
# Build for your current platform
npm run tauri build

# Or use our build script
chmod +x build-release.sh
./build-release.sh
```

The built application will be available in `src-tauri/target/release/bundle/`.

## Usage

### Interface Overview

XNote uses a three-pane layout:

1. **TagPane** (Left): Navigation and tag management
2. **NotePane** (Middle): List of notes
3. **ContentPane** (Right): Note content editing/display

### View Modes

- **View Mode** (👁️): Read-only rendered Markdown
- **Edit Mode** (✏️): Raw Markdown editing
- **Split Mode** (📱): Side-by-side editing and preview

### Core Features

#### Creating Notes
- Click the "+" button in the NotePane
- Notes are automatically saved as Markdown files

#### Tag Management
- Click the tag icon (🏷️) in the ActionBar
- Add, remove, or manage tags for notes

#### Search
- Use the search box in NotePane to find notes
- Search by title or content

#### Attachments
- Paste images directly into notes
- Drag and drop files to attach them

## Data Storage

### File Structure
```
mdnote/
├── config.json          # Application configuration
├── mdnote.db           # SQLite database (tags, metadata)
├── attachments/        # Attached files and images
└── *.md                # Markdown note files
```

### Configuration
The application creates a `config.json` file in your system's configuration directory:
- **Windows**: `%APPDATA%\MDNote\config.json`
- **macOS**: `~/Library/Application Support/MDNote/config.json`
- **Linux**: `~/.config/mdnote/config.json`

Default configuration:
```json
{
  "data_directory": "~/Documents/MDNote",
  "window_width": 1200,
  "window_height": 800,
  "sidebar_width": 240,
  "note_list_width": 320,
  "auto_save_interval": 5000,
  "theme": "light"
}
```

## Development

### Project Structure
```
xnote/
├── src/                    # Frontend Vue source code
│   ├── components/         # Vue components
│   ├── stores/             # Pinia stores
│   ├── types/              # TypeScript types
│   ├── utils/              # Utility functions
│   ├── App.vue             # Main application component
│   └── main.ts             # Application entry point
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/                # Rust source code
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── index.html              # HTML entry point
├── package.json            # Frontend dependencies
└── XNote需求.md            # Requirements document (Chinese)
```

### Backend Modules

1. **Config**: Application configuration management
2. **Database**: SQLite database operations
3. **Notes**: Note CRUD operations and file management
4. **Storage**: File system operations
5. **Tags**: Tag management and associations

### Running Tests

Frontend tests:
```bash
npm test
```

Backend tests:
```bash
cd src-tauri
cargo test
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/HuskarTang/xnote.git`
3. **Create** a feature branch: `git checkout -b feature/amazing-feature`
4. **Make** your changes and **test** them
5. **Commit** your changes: `git commit -m 'Add amazing feature'`
6. **Push** to the branch: `git push origin feature/amazing-feature`
7. **Open** a Pull Request

### Development Setup

```bash
# Install dependencies
npm install

# Start development server
npm run tauri dev

# Run tests
npm test                    # Frontend tests
cd src-tauri && cargo test  # Backend tests

# Build for production
./build-release.sh
```

### Reporting Issues

Found a bug? Have a feature request? Please [open an issue](https://github.com/HuskarTang/xnote/issues/new) with:

- 🐛 **Bug reports**: Steps to reproduce, expected vs actual behavior
- 💡 **Feature requests**: Clear description of the proposed feature
- 📚 **Documentation**: Improvements or corrections needed

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [**Tauri**](https://tauri.app/) - For the amazing desktop application framework
- [**Vue.js**](https://vuejs.org/) - For the reactive frontend framework
- [**Rust**](https://www.rust-lang.org/) - For the fast and safe backend
- [**Marked.js**](https://marked.js.org/) - For Markdown parsing
- [**Highlight.js**](https://highlightjs.org/) - For code syntax highlighting
- [**CodeMirror**](https://codemirror.net/) - For the code editor component

## 📊 Project Stats

![GitHub stars](https://img.shields.io/github/stars/HuskarTang/xnote?style=social)
![GitHub forks](https://img.shields.io/github/forks/HuskarTang/xnote?style=social)
![GitHub issues](https://img.shields.io/github/issues/HuskarTang/xnote)
![GitHub pull requests](https://img.shields.io/github/issues-pr/HuskarTang/xnote)

---

<div align="center">

**[⬆ Back to Top](#xnote)**

Made with ❤️ by the XNote team

</div>
