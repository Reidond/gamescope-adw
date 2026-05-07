# Gamescope GUI

A native GTK4/libadwaita wrapper for launching Steam games through Gamescope.

Use it in a Steam game's launch options:

```sh
gamescope-gui -- %command%
```

Environment variables placed before the wrapper are inherited by Gamescope and
the game:

```sh
MANGOHUD=1 gamescope-gui -- %command%
```

Everything after `--` is forwarded unchanged as the game command. The wrapper
shows a per-game settings window first, saves the selected Gamescope options,
then starts:

```sh
gamescope [saved options] -- [game command]
```

Profiles are stored under the XDG config directory:

```text
$XDG_CONFIG_HOME/gamescope-gui/profiles.toml
```

If `XDG_CONFIG_HOME` is not set, the standard fallback is:

```text
~/.config/gamescope-gui/profiles.toml
```

## Install

Install the latest release binary into `${XDG_BIN_DIR:-$HOME/.local/bin}`:

```sh
curl -fsSL https://raw.githubusercontent.com/Reidond/gamescope-adw/main/scripts/install.sh | bash
```

Other modes:

```sh
# Pinned version
./scripts/install.sh --version v0.1.0

# Rolling nightly build of main
./scripts/install.sh --nightly

# Custom install directory
./scripts/install.sh --bin-dir "$HOME/bin"
```

The script downloads the matching `x86_64-linux` archive from GitHub Releases
and places the `gamescope-gui` binary in the install directory. It warns if
that directory isn't on your `PATH`.

To remove the binary:

```sh
./scripts/uninstall.sh
```

Add `--purge` to also delete saved profiles under
`${XDG_CONFIG_HOME:-$HOME/.config}/gamescope-gui`.

## Build

Install the native development packages for GTK 4.14 or newer, libadwaita, GLib, Pango,
Cairo, GDK Pixbuf, Graphene, pkg-config, and Blueprint, plus the runtime
`gamescope` binary. Package names vary by distro; on Fedora the important
packages are typically:

```sh
sudo dnf install gtk4-devel libadwaita-devel blueprint-compiler gamescope pkgconf-pkg-config
```

Then build:

```sh
cargo build --release
```

For core tests that do not need GTK development packages:

```sh
cargo test --no-default-features
```

## Development

Use a temporary XDG config directory when testing profile behavior without
touching real saved profiles:

```sh
XDG_CONFIG_HOME="$(mktemp -d)" cargo run -- -- glxgears
```

For a repeatable local scratch profile store:

```sh
mkdir -p .run/xdg-config
XDG_CONFIG_HOME="$PWD/.run/xdg-config" cargo run -- -- glxgears
```
