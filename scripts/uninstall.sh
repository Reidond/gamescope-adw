#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="gamescope-gui"

bin_dir="${XDG_BIN_DIR:-$HOME/.local/bin}"
purge=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [--bin-dir <dir>] [--purge]

Removes the $BIN_NAME binary from \${XDG_BIN_DIR:-\$HOME/.local/bin}
(override with --bin-dir).

Options:
  --bin-dir <dir>   Override the install directory.
  --purge           Also remove saved profiles
                    (\${XDG_CONFIG_HOME:-\$HOME/.config}/$BIN_NAME).
  -h, --help        Show this help.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --bin-dir) bin_dir="${2:?--bin-dir requires a path}"; shift 2 ;;
        --purge)   purge=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

target="$bin_dir/$BIN_NAME"

if [ -e "$target" ] || [ -L "$target" ]; then
    rm -f "$target"
    echo "Removed $target"
else
    echo "Nothing to remove: $target not found"
fi

if [ "$purge" -eq 1 ]; then
    config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/$BIN_NAME"
    if [ -d "$config_dir" ]; then
        rm -rf "$config_dir"
        echo "Removed $config_dir"
    else
        echo "No profile data at $config_dir"
    fi
fi
