# ytmusic — Project Context

## What is this

A keyboard-driven terminal UI client for YouTube Music, built in Rust. Targets developers and terminal enthusiasts who want a fast, no-nonsense music player that runs in any terminal.

## Tech stack

- **Language:** Rust (2021 edition)
- **TUI:** ratatui 0.29 + crossterm 0.28
- **Async:** tokio (full features)
- **HTTP:** reqwest with rustls TLS (no OpenSSL dependency — critical for cross-compilation)
- **Audio:** mpv subprocess controlled via JSON IPC over Unix sockets (macOS/Linux) or named pipes (Windows)
- **Stream extraction:** yt-dlp (called internally by mpv)
- **YouTube Music data:** InnerTube API (`/youtubei/v1/search` with WEB_REMIX client context)

## Architecture

```
main.rs        → terminal setup, event loop (tokio::select! over crossterm EventStream + tick timer)
app.rs         → AppEvent enum, App struct (all state), tick() drains events + processes pending loads
api.rs         → YtMusicClient (InnerTube POST requests), deeply nested JSON response parsing
player.rs      → MpvProcess (subprocess + IPC), PlayerSender (Clone-able command channel), platform-conditional (cfg unix/windows)
input.rs       → Mode-based key dispatch (Normal / Search), vim-style bindings
ui/mod.rs      → ratatui immediate-mode rendering: library panel, search results, queue, player bar, help overlay
ui/theme.rs    → color constants, style helpers
```

**Key design decisions:**
- mpv is controlled via IPC (not libmpv C bindings) — avoids C dependency, simpler cross-compilation
- Search runs in a background tokio task, results sent back via AppEvent channel — UI never blocks
- PlayerSender is Clone so commands can be sent without owning MpvProcess
- pending_load pattern: track-end handler sets a URL, tick() sends it to mpv — avoids async in sync event handlers

## Distribution

- **GitHub Actions CI:** lint (clippy + fmt) + build on ubuntu/macos/windows on every push
- **GitHub Actions Release:** on `git tag v*`, builds 5 binaries (linux x86/arm64, macos x86/arm64, windows x86), creates GH Release with checksums
- **install.sh:** curl one-liner that detects OS/arch, downloads binary, auto-installs mpv+yt-dlp via detected package manager
- **Homebrew formula:** `Formula/ytmusic.rb` with `depends_on "mpv"` and `depends_on "yt-dlp"`
- **crates.io:** package name is `ytmusic-tui`, binary name is `ytmusic`

## Dependencies (runtime)

Users need `mpv` and `yt-dlp` installed. The app checks at startup and prints install instructions if missing.

## Competitors

See `.context/competitor-analysis.md` for detailed comparison with ytermusic, youtui, ytui-music, and youtui-player. Key gaps we need to close: config file, theming, offline caching, album art, lyrics, media keys.

## Build

```sh
cargo build --release
./target/release/ytmusic
```

## Release workflow

1. Bump version in Cargo.toml
2. `git tag vX.Y.Z && git push origin main --tags`
3. GH Actions builds + publishes automatically
4. Update sha256 in Formula/ytmusic.rb from release checksums

## Code style

- No comments except for genuinely complex logic
- Self-documenting code with clear names
- Small, focused changes
- `cargo fmt` and `cargo clippy -- -D warnings` must pass (enforced in CI)
