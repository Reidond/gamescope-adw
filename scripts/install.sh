#!/usr/bin/env bash
set -euo pipefail

REPO="Reidond/gamescope-adw"
BIN_NAME="gamescope-gui"
ARCH_TRIPLE="x86_64-linux"

bin_dir="${XDG_BIN_DIR:-$HOME/.local/bin}"
release="latest"

usage() {
    cat <<EOF
Usage: $(basename "$0") [--nightly | --version <tag>] [--bin-dir <dir>]

Downloads a $BIN_NAME release archive and installs the binary to
\${XDG_BIN_DIR:-\$HOME/.local/bin} (override with --bin-dir).

Options:
  --nightly             Install the rolling nightly build.
  --version <tag>       Install a specific tag (e.g. v0.1.0).
  --bin-dir <dir>       Override the install directory.
  -h, --help            Show this help.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --nightly) release="nightly"; shift ;;
        --version) release="${2:?--version requires a tag}"; shift 2 ;;
        --bin-dir) bin_dir="${2:?--bin-dir requires a path}"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if command -v curl >/dev/null 2>&1; then
    fetch()    { curl --fail --location --silent --show-error "$1"; }
    fetch_to() { curl --fail --location --silent --show-error --output "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    fetch()    { wget -qO- "$1"; }
    fetch_to() { wget -qO "$2" "$1"; }
else
    echo "error: curl or wget is required" >&2
    exit 1
fi

if [ "$release" = "latest" ]; then
    echo "Looking up latest release..."
    api_url="https://api.github.com/repos/$REPO/releases/latest"
else
    api_url="https://api.github.com/repos/$REPO/releases/tags/$release"
fi

if ! release_json=$(fetch "$api_url" 2>/dev/null); then
    case "$release" in
        latest)
            echo "error: no stable release found for $REPO." >&2
            echo "       Try --nightly for the rolling pre-release build," >&2
            echo "       or --version <tag> for a specific tag." >&2
            ;;
        *)
            echo "error: release '$release' not found at $api_url" >&2
            ;;
    esac
    exit 1
fi

tag=$(printf '%s' "$release_json" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
if [ -z "${tag:-}" ]; then
    echo "error: failed to resolve release tag from $api_url" >&2
    exit 1
fi

asset_url=$(printf '%s' "$release_json" \
    | sed -n 's|.*"browser_download_url": *"\([^"]*'"$ARCH_TRIPLE"'\.tar\.gz\)".*|\1|p' \
    | head -n1)
if [ -z "${asset_url:-}" ]; then
    echo "error: no $ARCH_TRIPLE asset found in release $tag" >&2
    exit 1
fi

echo "Installing $BIN_NAME ($tag) to $bin_dir"

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

archive="$tmpdir/${BIN_NAME}.tar.gz"
echo "Downloading $asset_url"
fetch_to "$asset_url" "$archive"

tar -xzf "$archive" -C "$tmpdir"

if [ ! -x "$tmpdir/$BIN_NAME" ]; then
    echo "error: archive did not contain executable $BIN_NAME" >&2
    exit 1
fi

mkdir -p "$bin_dir"
install -m 0755 "$tmpdir/$BIN_NAME" "$bin_dir/$BIN_NAME"

echo "Installed: $bin_dir/$BIN_NAME"

case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *)
        echo
        echo "note: $bin_dir is not in your PATH."
        echo "      Add it to your shell profile to run $BIN_NAME directly:"
        echo "          export PATH=\"$bin_dir:\$PATH\""
        ;;
esac
