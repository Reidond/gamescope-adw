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

## Build

Install the native development packages for GTK4, libadwaita, GLib, Pango,
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
