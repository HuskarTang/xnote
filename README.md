# MDNote

A lightweight, cross-platform Markdown note-taking application built with Rust, Tauri, and Vue 3.

![MDNote Screenshot](docs/screenshot.png)

## Features

### 📝 Markdown Support
- **Full Markdown syntax** support with live preview
- **Rich text editing** with syntax highlighting
- **Split view mode** for simultaneous editing and preview
- **Image paste support** with automatic attachment management
- **Code syntax highlighting** for multiple languages

### 🏷️ Organization
- **Tag system** for categorizing notes
- **Favorites** for quick access to important notes
- **Search functionality** across all notes and tags
- **Trash/Recycle bin** for deleted notes
- **Untagged notes** view for better organization

### 💾 Storage
- **File-based storage** - your notes are stored as `.md` files
- **SQLite database** for metadata and tag relationships
- **Attachments folder** for images and other files
- **Configurable data directory** - choose where to store your notes

### 🎨 Interface
- **Three-pane layout**: Tags | Notes List | Editor/Viewer
- **Resizable panels** with customizable widths (default ratio 2:3:12)
- **Multiple view modes**: View, Edit, and Split view
- **Auto-save** functionality
- **Cross-platform** support (Windows, macOS, Linux)

### 🔧 Advanced Features
- **Export notes** to HTML, PDF, or Markdown
- **Attachment management** for images and files
- **Keyboard shortcuts** for efficient workflow
- **Auto-save** with configurable intervals
- **First-run setup** for data directory configuration

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/mdnote/mdnote/releases) page:

- **Windows**: `MDNote_x.x.x_x64_en-US.msi`
- **macOS**: `MDNote_x.x.x_x64.dmg`
- **Linux**: `MDNote_x.x.x_amd64.AppImage`

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v16 or later)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

#### Build Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/mdnote/mdnote.git
   cd mdnote
   ```

2. **Install dependencies**
   ```bash
   # Install Node.js dependencies
   npm install
   
   # Install Rust dependencies (handled by Cargo)
   ```

3. **Development mode**
   ```bash
   npm run tauri dev
   ```

4. **Build for production**
   ```bash
   npm run tauri build
   ```

The built application will be available in `src-tauri/target/release/bundle/`.

## Usage

### First Run

When you first launch MDNote, you'll be prompted to select a data directory where your notes will be stored. This directory will contain:

- `*.md` files - your notes in Markdown format
- `attachments/` - folder for images and file attachments
- `mdnote.db` - SQLite database for tags and metadata
- `config.json` - application configuration

### Creating Notes

1. Click the **"+ New Note"** button in the notes panel
2. Start typing in the editor
3. The note title is automatically generated from the first heading
4. Notes are auto-saved as you type

### Managing Tags

1. Click the **🏷️ tag button** in the editor toolbar
2. Add new tags or select from existing ones
3. Remove tags by clicking the × next to them
4. Use the sidebar to filter notes by tags

### View Modes

- **View Mode** (👁️): Read-only rendered Markdown
- **Edit Mode** (✏️): Raw Markdown editing
- **Split Mode** (📱): Side-by-side editing and preview

### Keyboard Shortcuts

- `Ctrl/Cmd + S` - Save note
- `Ctrl/Cmd + N` - New note
- `Ctrl/Cmd + F` - Search notes
- `Delete` - Move note to trash

## Configuration

The application stores its configuration in:
- **Windows**: `%APPDATA%/mdnote/config.json`
- **macOS**: `~/Library/Application Support/mdnote/config.json`
- **Linux**: `~/.config/mdnote/config.json`

### Configuration Options

```json
{
  "data_directory": "/path/to/your/notes",
  "window_width": 1200,
  "window_height": 800,
  "sidebar_width": 250,
  "note_list_width": 350,
  "auto_save_interval": 5000,
  "theme": "light"
}
```

## File Structure

```
your-data-directory/
├── attachments/           # Images and file attachments
│   ├── image-1234567890.png
│   └── document.pdf
├── mdnote.db             # SQLite database for metadata
├── note1.md              # Your markdown notes
├── note2.md
└── ...
```

## Development

### Project Structure

```
mdnote/
├── src/                  # Rust backend (Tauri)
│   ├── commands/         # Tauri command handlers
│   ├── database.rs       # SQLite database operations
│   ├── models.rs         # Data models
│   ├── config.rs         # Configuration management
│   └── main.rs           # Application entry point
├── src/                  # Vue 3 frontend
│   ├── components/       # Vue components
│   ├── stores/           # Pinia stores
│   ├── views/            # Page components
│   └── main.ts           # Frontend entry point
├── Cargo.toml            # Rust dependencies
├── package.json          # Node.js dependencies
└── tauri.conf.json       # Tauri configuration
```

### Tech Stack

- **Backend**: Rust with Tauri framework
- **Frontend**: Vue 3 with TypeScript
- **State Management**: Pinia
- **Database**: SQLite with sqlx
- **Editor**: CodeMirror 6
- **Markdown**: marked.js with DOMPurify
- **Styling**: SCSS

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Notable](https://notable.app/) - an excellent note-taking application
- Built with [Tauri](https://tauri.app/) - a framework for building desktop applications
- Uses [CodeMirror](https://codemirror.net/) for the markdown editor
- Markdown rendering by [marked.js](https://marked.js.org/)

## Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/mdnote/mdnote/issues) page
2. Create a new issue with detailed information
3. Join our [Discussions](https://github.com/mdnote/mdnote/discussions) for community support

---

**MDNote** - Simple, powerful, and yours. 📝