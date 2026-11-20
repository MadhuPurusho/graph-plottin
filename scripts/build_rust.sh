#!/usr/bin/env bash
#
# build_rust.sh — Build the graph-plotting-core Rust crate and generate
#                 UniFFI Swift bindings + XCFramework for the macOS app.
#
# Usage:  ./scripts/build_rust.sh          (from the repo root)
#         ./scripts/build_rust.sh --clean   (clean build)
#
set -euo pipefail

# ── Resolve paths ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$REPO_ROOT/rust/graph-plotting-core"
SWIFT_BRIDGE_DIR="$REPO_ROOT/macos/GraphPlottingApp/Sources/Bridge/Generated"
FFI_PKG_DIR="$REPO_ROOT/GraphPlottingCoreFFI"
XCFRAMEWORK_DIR="$FFI_PKG_DIR/GraphPlottingCoreFFI.xcframework"

TARGET="aarch64-apple-darwin"
LIB_NAME="libgraph_plotting_core.a"

# ── Source Cargo environment ─────────────────────────────────────────
if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi

if ! command -v cargo &>/dev/null; then
    echo "❌  cargo not found. Install Rust via https://rustup.rs"
    exit 1
fi

if ! command -v xcrun &>/dev/null; then
    echo "❌  xcrun not found. Install Xcode command-line tools."
    exit 1
fi

# ── Optional clean ───────────────────────────────────────────────────
if [[ "${1:-}" == "--clean" ]]; then
    echo "🧹  Cleaning previous build artifacts…"
    (cd "$RUST_DIR" && cargo clean)
    rm -rf "$XCFRAMEWORK_DIR"
    rm -rf "$SWIFT_BRIDGE_DIR"
fi

# ── 1. Build the Rust static library (release) ──────────────────────
echo ""
echo "🔨  Building $LIB_NAME for $TARGET (release)…"
(cd "$RUST_DIR" && cargo build --release --target "$TARGET")

STATIC_LIB="$RUST_DIR/target/$TARGET/release/$LIB_NAME"
if [[ ! -f "$STATIC_LIB" ]]; then
    echo "❌  Static library not found at $STATIC_LIB"
    echo "    Check that crate-type includes 'staticlib' in Cargo.toml."
    exit 1
fi
echo "✅  Static library built: $STATIC_LIB"

# ── 2. Generate UniFFI Swift bindings ────────────────────────────────
echo ""
echo "📦  Generating UniFFI Swift bindings…"
mkdir -p "$SWIFT_BRIDGE_DIR"

(cd "$RUST_DIR" && cargo run --release --features uniffi/cli -- \
    generate \
    --library "target/$TARGET/release/$LIB_NAME" \
    --language swift \
    --out-dir "$SWIFT_BRIDGE_DIR")

# Check that at least one .swift file was generated
SWIFT_COUNT=$(find "$SWIFT_BRIDGE_DIR" -name '*.swift' 2>/dev/null | wc -l | tr -d ' ')
if [[ "$SWIFT_COUNT" -eq 0 ]]; then
    echo "❌  No Swift files generated in $SWIFT_BRIDGE_DIR"
    echo "    Verify that the crate exposes uniffi types."
    exit 1
fi
echo "✅  Generated $SWIFT_COUNT Swift binding file(s) in $SWIFT_BRIDGE_DIR"

# Ensure Swift bindings import GraphPlottingCoreFFI
for sf in "$SWIFT_BRIDGE_DIR"/*.swift; do
    if ! grep -q "import GraphPlottingCoreFFI" "$sf"; then
        sed -i '' 's/#if canImport(graph_plotting_coreFFI)/#if canImport(GraphPlottingCoreFFI)\
import GraphPlottingCoreFFI\
#endif\
#if canImport(graph_plotting_coreFFI)/g' "$sf"
    fi
done

# ── 3. Create XCFramework ───────────────────────────────────────────
echo ""
echo "📐  Creating XCFramework…"
rm -rf "$XCFRAMEWORK_DIR"

# Create a temporary headers directory with the generated C header
HEADER_DIR="$(mktemp -d)"
MODULE_DIR="$HEADER_DIR"
cp "$SWIFT_BRIDGE_DIR"/*.h "$HEADER_DIR/" 2>/dev/null || true

# Generate modulemap
HEADER_FILES=$(find "$HEADER_DIR" -name '*.h' -exec basename {} \;)
cat > "$MODULE_DIR/module.modulemap" <<EOF
module GraphPlottingCoreFFI {
$(for h in $HEADER_FILES; do echo "    header \"$h\""; done)
    export *
}
EOF

xcodebuild -create-xcframework \
    -library "$STATIC_LIB" \
    -headers "$HEADER_DIR" \
    -output "$XCFRAMEWORK_DIR"

rm -rf "$HEADER_DIR"

if [[ ! -d "$XCFRAMEWORK_DIR" ]]; then
    echo "❌  Failed to create XCFramework."
    exit 1
fi
echo "✅  XCFramework created: $XCFRAMEWORK_DIR"

# ── 4. Summary ──────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  Build complete!"
echo ""
echo "  Static lib : $STATIC_LIB"
echo "  Bindings   : $SWIFT_BRIDGE_DIR"
echo "  XCFramework: $XCFRAMEWORK_DIR"
echo "════════════════════════════════════════════════════════════════"
