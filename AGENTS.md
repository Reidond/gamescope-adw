# AGENTS

Run the app with:

```sh
cargo run -- -- glxgears
```

Use `gamescope-gui -- %command%` for Steam launch options.

UI rules:
- Define UI in Blueprint.
- Follow GNOME HIG strictly.

After any Rust code change, run:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

Both must pass before declaring the change complete; CI enforces them.
