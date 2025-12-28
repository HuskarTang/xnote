#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "🚀 Starting XNote release build..."

# 1. Clean previous build artifacts
echo "🧹 Cleaning up old build artifacts..."
rm -rf src-tauri/target/release

# 2. Install frontend dependencies
echo "📦 Installing npm dependencies..."
npm install

# 3. Build the Tauri application
echo "🛠️ Building the application..."
npm run tauri build

echo "✅ Build complete!"
echo "📦 Your application bundle can be found in the 'src-tauri/target/release/bundle/' directory."

exit 0
