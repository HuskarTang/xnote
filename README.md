# XNote

XNote is a cross-platform lightweight Markdown note-taking application built with Vue 3, TypeScript, and Tauri. It provides a rich text editing experience with support for Markdown syntax, image pasting, and tag-based organization.

![XNote Screenshot](attachments/Clipboard_2025-09-05-16-23-00.png)

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

## Installation

### Prerequisites

- Node.js (v16 or higher)
- Rust (via rustup)
- Tauri CLI

### Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd xnote
```

2. Install frontend dependencies:
```bash
npm install
```

3. Install Tauri CLI:
```bash
npm install -g @tauri-apps/cli
```

### Development

Start the development server:
```bash
npm run tauri dev
```

### Building

Build the application for production:
```bash
npm run tauri build
```

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

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Tauri](https://tauri.studio/) for the desktop application framework
- [Vue.js](https://vuejs.org/) for the frontend framework
- [Marked.js](https://marked.js.org/) for Markdown parsing
- [Highlight.js](https://highlightjs.org/) for code syntax highlighting
