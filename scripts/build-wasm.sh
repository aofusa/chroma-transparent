#!/bin/bash
#
# WASM ビルドスクリプト
#
# 使用方法:
#   ./scripts/build-wasm.sh
#   ./scripts/build-wasm.sh --release  (最適化ビルド)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/static"

# 引数チェック
RELEASE_FLAG=""
OPT_LEVEL="dev"
if [[ "$1" == "--release" ]] || [[ "$1" == "-r" ]]; then
    RELEASE_FLAG="--release"
    OPT_LEVEL="release"
    echo "Building in release mode..."
else
    echo "Building in dev mode (use --release for optimized build)..."
fi

# wasm-packがインストールされているか確認
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed."
    echo "Install with: cargo install wasm-pack"
    exit 1
fi

# ビルドディレクトリに移動
cd "$PROJECT_DIR"

echo ""
echo "=== Step 1: Building WASM ==="
wasm-pack build --target web --out-dir pkg $RELEASE_FLAG -- --no-default-features --features wasm

echo ""
echo "=== Step 2: Preparing static files ==="
mkdir -p "$OUTPUT_DIR"

# WASMファイルをコピー
cp pkg/chroma_transparent_bg.wasm "$OUTPUT_DIR/"
cp pkg/chroma_transparent.js "$OUTPUT_DIR/"

# アセットをコピー
cp assets/index-wasm.html "$OUTPUT_DIR/index.html"
cp assets/style.css "$OUTPUT_DIR/"
cp assets/wasm.js "$OUTPUT_DIR/app.js"

# リリースビルドの場合はwasm-optで最適化
if [[ "$OPT_LEVEL" == "release" ]] && command -v wasm-opt &> /dev/null; then
    echo ""
    echo "=== Step 3: Optimizing WASM with wasm-opt ==="
    wasm-opt -Os -o "$OUTPUT_DIR/chroma_transparent_bg.wasm" "$OUTPUT_DIR/chroma_transparent_bg.wasm"
fi

# ファイルサイズを表示
echo ""
echo "=== Build complete ==="
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "Files:"
ls -lh "$OUTPUT_DIR"

echo ""
echo "WASM size:"
ls -lh "$OUTPUT_DIR/chroma_transparent_bg.wasm" | awk '{print $5}'

echo ""
echo "To serve locally, run:"
echo "  python3 -m http.server 8000 --directory $OUTPUT_DIR"
echo "  # or"
echo "  npx serve $OUTPUT_DIR"

