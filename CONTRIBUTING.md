# Contributing to MDNote

Thank you for your interest in contributing to MDNote! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

1. **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
2. **Node.js** (v16 or later) - [Install Node.js](https://nodejs.org/)
3. **Git** - [Install Git](https://git-scm.com/)

### Platform-specific Requirements

#### Windows
- Microsoft Visual Studio C++ Build Tools
- WebView2 (usually pre-installed on Windows 10/11)

#### macOS
- Xcode Command Line Tools: `xcode-select --install`

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Getting Started

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/mdnote.git
   cd mdnote
   ```

2. **Install Dependencies**
   ```bash
   npm install
   ```

3. **Run Development Server**
   ```bash
   # Using npm scripts
   npm run tauri dev
   
   # Or using our helper scripts
   ./scripts/dev.sh      # Linux/macOS
   scripts\dev.bat       # Windows
   ```

## Project Structure

```
mdnote/
├── src/                  # Vue 3 frontend
│   ├── components/       # Reusable Vue components
│   ├── stores/           # Pinia state management
│   ├── views/            # Page components
│   ├── styles/           # SCSS stylesheets
│   └── types/            # TypeScript type definitions
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── commands/     # Tauri command handlers
│   │   ├── database.rs   # SQLite operations
│   │   ├── models.rs     # Data models
│   │   └── main.rs       # Application entry point
│   └── Cargo.toml        # Rust dependencies
├── scripts/              # Build and development scripts
└── docs/                 # Documentation and assets
```

## Development Guidelines

### Code Style

#### Rust
- Follow standard Rust formatting with `rustfmt`
- Use `clippy` for linting
- Write documentation for public APIs
- Use meaningful variable and function names

#### TypeScript/Vue
- Use TypeScript for all new code
- Follow Vue 3 Composition API patterns
- Use Pinia for state management
- Write component documentation

#### SCSS
- Use the existing variable system in `src/styles/variables.scss`
- Follow BEM naming convention for CSS classes
- Keep styles scoped to components when possible

### Commit Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add export to PDF functionality
fix: resolve auto-save timing issue
docs: update installation instructions
```

### Testing

Currently, the project focuses on manual testing. When adding new features:

1. Test on multiple platforms (Windows, macOS, Linux)
2. Test edge cases (empty notes, large files, special characters)
3. Verify database operations work correctly
4. Check that the UI remains responsive

### Pull Request Process

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Write clean, documented code
   - Test your changes thoroughly
   - Update documentation if needed

3. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

4. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a Pull Request on GitHub.

5. **PR Requirements**
   - Clear description of changes
   - Screenshots for UI changes
   - Testing instructions
   - Link to related issues

## Feature Requests and Bug Reports

### Bug Reports

When reporting bugs, please include:

1. **Environment Information**
   - Operating System and version
   - MDNote version
   - Steps to reproduce

2. **Expected vs Actual Behavior**
   - What you expected to happen
   - What actually happened
   - Screenshots if applicable

3. **Additional Context**
   - Error messages
   - Log files (if available)
   - Related configuration

### Feature Requests

For feature requests, please provide:

1. **Use Case**: Why is this feature needed?
2. **Proposed Solution**: How should it work?
3. **Alternatives**: Any alternative solutions considered?
4. **Additional Context**: Screenshots, mockups, or examples

## Architecture Decisions

### Why Tauri?
- Cross-platform desktop applications with web technologies
- Smaller bundle size compared to Electron
- Better security model
- Native performance for system operations

### Why Vue 3?
- Excellent TypeScript support
- Composition API for better code organization
- Reactive state management with Pinia
- Great developer experience

### Why SQLite?
- Lightweight and embedded
- No external database server required
- ACID compliance for data integrity
- Excellent Rust support with sqlx

## Getting Help

- **GitHub Discussions**: For questions and community support
- **GitHub Issues**: For bug reports and feature requests
- **Documentation**: Check the README and inline code documentation

## Recognition

Contributors will be recognized in:
- The project's README
- Release notes for significant contributions
- GitHub's contributor graphs

Thank you for contributing to MDNote! 🎉
