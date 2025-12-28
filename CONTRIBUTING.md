# Contributing to XNote

Thank you for your interest in contributing to XNote! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Node.js (v16 or higher)
- Rust (latest stable)
- Git

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/HuskarTang/xnote.git
   cd xnote
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Start the development server:
   ```bash
   npm run tauri dev
   ```

## 🐛 Reporting Issues

### Bug Reports

When reporting bugs, please include:

- **Description**: Clear description of the issue
- **Steps to reproduce**: Detailed steps to reproduce the bug
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Environment**: OS, Node.js version, Rust version
- **Screenshots**: If applicable

### Feature Requests

When requesting features, please include:

- **Description**: Clear description of the feature
- **Use case**: Why this feature would be useful
- **Implementation ideas**: If you have any thoughts on implementation

## 💻 Development Guidelines

### Code Style

- **Frontend**: Follow Vue.js and TypeScript best practices
- **Backend**: Follow Rust conventions and use `cargo fmt`
- **Commits**: Use conventional commit messages

### Testing

- Write tests for new features
- Ensure all tests pass before submitting PR
- Run both frontend and backend tests:
  ```bash
  npm test                    # Frontend
  cd src-tauri && cargo test  # Backend
  ```

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Add tests if applicable
4. Update documentation if needed
5. Ensure all tests pass
6. Submit a pull request

### Commit Message Format

Use conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

## 📁 Project Structure

```
xnote/
├── src/                    # Frontend Vue.js code
│   ├── components/         # Vue components
│   ├── stores/             # Pinia stores
│   ├── types/              # TypeScript types
│   └── utils/              # Utility functions
├── src-tauri/              # Backend Rust code
│   ├── src/                # Rust source code
│   │   ├── config/         # Configuration management
│   │   ├── notes/          # Note management
│   │   ├── storage/        # File storage
│   │   └── tags/           # Tag management
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
└── build-release.sh        # Build script
```

## 🔧 Building and Testing

### Development Build

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
# or
./build-release.sh
```

### Running Tests

```bash
# Frontend tests
npm test

# Backend tests
cd src-tauri
cargo test

# All tests
npm run test:all
```

## 📝 Documentation

- Update README.md for user-facing changes
- Add inline code comments for complex logic
- Update this CONTRIBUTING.md for process changes

## 🎯 Areas for Contribution

We welcome contributions in these areas:

- **Bug fixes**: Help us squash bugs
- **Features**: Implement new functionality
- **Documentation**: Improve docs and examples
- **Testing**: Add test coverage
- **Performance**: Optimize code performance
- **UI/UX**: Improve user interface and experience

## 📞 Getting Help

- **Issues**: [GitHub Issues](https://github.com/HuskarTang/xnote/issues)
- **Discussions**: [GitHub Discussions](https://github.com/HuskarTang/xnote/discussions)

## 📄 License

By contributing to XNote, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing! 🎉
