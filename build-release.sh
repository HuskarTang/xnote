#!/bin/bash

# XNote Release Build Script
# This script builds XNote for different platforms

set -e

echo "🚀 Starting XNote Release Build Process..."

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed"
    exit 1
fi
echo "✅ Node.js version: $(node --version)"

# Check npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed"
    exit 1
fi
echo "✅ npm version: $(npm --version)"

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed"
    exit 1
fi
echo "✅ Rust version: $(rustc --version)"

# Check Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed"
    exit 1
fi
echo "✅ Cargo version: $(cargo --version)"

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🏗️ Building frontend..."
npm run build

# Build Tauri app (without bundling first to test compilation)
echo "🔧 Testing Rust compilation..."
cd src-tauri
cargo build --release
cd ..

echo "✅ Rust compilation successful!"

# Check if we're on macOS or Windows for platform-specific builds
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 Detected macOS - Building macOS app..."
    # For now, just build the binary without bundling due to icon issues
    echo "✅ macOS binary built successfully at: src-tauri/target/release/XNote"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo "🪟 Detected Windows - Building Windows app..."
    npm run tauri build
else
    echo "🐧 Detected Linux - Building Linux app..."
    npm run tauri build
fi

echo "🎉 Build process completed!"
echo "📁 Build artifacts can be found in:"
echo "   - Binary: src-tauri/target/release/"
echo "   - Bundled app: src-tauri/target/release/bundle/"