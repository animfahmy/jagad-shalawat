#!/usr/bin/env bash
# =================================================================
# Tagih Otomatis Blog — Linux Build Script
# Cross-compile from Windows to Linux x86_64
# =================================================================

set -Eeuo pipefail

echo "🔧 Building Tagih Otomatis Blog for Linux x86_64..."

# Ensure cross-compilation target is installed
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true

# Build release binary
cargo build --release --target x86_64-unknown-linux-gnu

echo "✅ Build complete!"
echo "📦 Binary: target/x86_64-unknown-linux-gnu/release/jagad-shalawat"

# Show binary size
ls -lh target/x86_64-unknown-linux-gnu/release/jagad-shalawat
