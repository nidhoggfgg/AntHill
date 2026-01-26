#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Package atom_node with uv and a bundled Python, then zip it.

Usage:
  scripts/package_bundle.sh [--python-version X] [--output-dir DIR] [--target TARGET] [--skip-build]

Options:
  --python-version X  Python version to bundle (default: 3.12 or $PYTHON_VERSION)
  --output-dir DIR    Output directory for the bundle (default: ./dist)
  --target TARGET     Cargo target triple for cross builds
  --skip-build        Skip cargo build step
  -h, --help          Show this help
EOF
}

PYTHON_VERSION="${PYTHON_VERSION:-3.12}"
OUTPUT_DIR=""
CARGO_TARGET=""
SKIP_BUILD=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --python-version)
      PYTHON_VERSION="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --target)
      CARGO_TARGET="$2"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$ROOT_DIR/Cargo.toml"

read_cargo_field() {
  local key="$1"
  awk -F= -v key="$key" '
    $0 ~ /^\[package\]/ {in_pkg=1; next}
    in_pkg && $0 ~ /^\[/ {in_pkg=0}
    in_pkg && $1 ~ "^[[:space:]]*" key "[[:space:]]*$" {
      val=$2
      sub(/^[[:space:]]*/, "", val)
      sub(/[[:space:]]*$/, "", val)
      gsub(/"/, "", val)
      print val
      exit
    }
  ' "$CARGO_TOML"
}

if [[ ! -f "$CARGO_TOML" ]]; then
  echo "Cargo.toml not found at $CARGO_TOML" >&2
  exit 1
fi

NAME="$(read_cargo_field name)"
VERSION="$(read_cargo_field version)"

if [[ -z "$NAME" || -z "$VERSION" ]]; then
  echo "Failed to read package name/version from $CARGO_TOML" >&2
  exit 1
fi

OS_RAW="$(uname -s)"
case "$OS_RAW" in
  Linux) OS="linux" ;;
  Darwin) OS="darwin" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *) OS="$(echo "$OS_RAW" | tr '[:upper:]' '[:lower:]')" ;;
esac

ARCH_RAW="$(uname -m)"
case "$ARCH_RAW" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) ARCH="$ARCH_RAW" ;;
esac

BUNDLE_NAME="${NAME}-${VERSION}-${OS}-${ARCH}"
DIST_DIR="${OUTPUT_DIR:-$ROOT_DIR/dist}"
BUNDLE_DIR="$DIST_DIR/$BUNDLE_NAME"

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  if [[ -n "$CARGO_TARGET" ]]; then
    cargo build --release --target "$CARGO_TARGET"
  else
    cargo build --release
  fi
fi

if [[ -n "$CARGO_TARGET" ]]; then
  BIN_DIR="$ROOT_DIR/target/$CARGO_TARGET/release"
else
  BIN_DIR="$ROOT_DIR/target/release"
fi

BIN_PATH="$BIN_DIR/$NAME"
if [[ "$OS" == "windows" ]]; then
  BIN_PATH="${BIN_PATH}.exe"
fi

if [[ ! -f "$BIN_PATH" ]]; then
  echo "Binary not found: $BIN_PATH" >&2
  exit 1
fi

UV_BIN="$(command -v uv || true)"
if [[ -z "$UV_BIN" ]]; then
  echo "uv not found in PATH. Install uv first." >&2
  exit 1
fi

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/bin" "$BUNDLE_DIR/python"

cp "$BIN_PATH" "$BUNDLE_DIR/bin/$NAME"
cp "$UV_BIN" "$BUNDLE_DIR/bin/uv"

uv python install --install-dir "$BUNDLE_DIR/python" "$PYTHON_VERSION"

PYTHON_BIN="$(find "$BUNDLE_DIR/python" -type f \
  \( -path "*/bin/python3.[0-9]*" -o -path "*/bin/python3" -o -path "*/bin/python" -o -path "*/Scripts/python.exe" \) \
  ! -name "*-config" | sort | head -n1)"

if [[ -z "$PYTHON_BIN" ]]; then
  PYTHON_BIN="$(find "$BUNDLE_DIR/python" -type l \
    \( -path "*/bin/python3" -o -path "*/bin/python" \) | sort | head -n1)"
fi

if [[ -z "$PYTHON_BIN" ]]; then
  echo "Bundled Python executable not found under $BUNDLE_DIR/python" >&2
  exit 1
fi

PYTHON_REL="${PYTHON_BIN#"$BUNDLE_DIR"/}"
echo "$PYTHON_REL" > "$BUNDLE_DIR/PYTHON_PATH"

cat > "$BUNDLE_DIR/$NAME" <<EOF
#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)
PYTHON_REL=\$(cat "\$ROOT_DIR/PYTHON_PATH")
PYTHON_BIN="\$ROOT_DIR/\$PYTHON_REL"

if [[ ! -x "\$PYTHON_BIN" ]]; then
  echo "Bundled python not found or not executable: \$PYTHON_BIN" >&2
  exit 1
fi

export PATH="\$ROOT_DIR/bin:\$PATH"
export UV_PYTHON="\$PYTHON_BIN"

exec "\$ROOT_DIR/bin/$NAME" "\$@"
EOF

chmod +x "$BUNDLE_DIR/$NAME"

ZIP_PATH="$DIST_DIR/$BUNDLE_NAME.zip"
rm -f "$ZIP_PATH"

if command -v zip >/dev/null 2>&1; then
  (cd "$DIST_DIR" && zip -r "$ZIP_PATH" "$BUNDLE_NAME")
else
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$DIST_DIR" "$BUNDLE_NAME" "$ZIP_PATH" <<'PY'
import os
import sys
import zipfile

dist_dir, bundle_name, zip_path = sys.argv[1:4]
bundle_dir = os.path.join(dist_dir, bundle_name)

with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    for root, _, files in os.walk(bundle_dir):
        for file_name in files:
            full_path = os.path.join(root, file_name)
            rel_path = os.path.relpath(full_path, dist_dir)
            zf.write(full_path, rel_path)
PY
  else
    echo "zip not found and python3 is unavailable to create the archive." >&2
    exit 1
  fi
fi

echo "Bundle created:"
echo "  $BUNDLE_DIR"
echo "  $ZIP_PATH"
