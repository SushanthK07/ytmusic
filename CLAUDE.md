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
- **Lyrics:** LRCLIB API (`lrclib.net/api/get`) — free, no auth, supports synced LRC format
- **Config:** TOML via `toml` crate, stored at `~/.config/ytmusic/config.toml`
- **Storage:** JSON via `serde_json`, stored at `~/.config/ytmusic/` (favorites, playlists, queue, history)
- **Cache:** Audio files cached at `~/.cache/ytmusic/` with LRU eviction; background `yt-dlp` downloads

## Architecture

```
main.rs        → terminal setup, event loop (tokio::select! over crossterm EventStream + tick timer), mouse capture
app.rs         → AppEvent enum, App struct (all state), tick() drains events + processes pending loads + gapless prefetch
api.rs         → YtMusicClient (InnerTube POST requests + LRCLIB lyrics + browse/explore), deeply nested JSON response parsing
player.rs      → MpvProcess (subprocess + IPC), PlayerSender (Clone-able command channel), platform-conditional (cfg unix/windows)
input.rs       → Mode-based key dispatch (Normal / Search / Explore), context handlers (Settings, Playlists, PlaylistPicker), mouse handler
config.rs      → TOML config loading/saving, Theme struct (12 presets + hex overrides), KeyBindings system (HashMap<Action, Vec<KeyBinding>>)
cache.rs       → AudioCache with CacheIndex (JSON at ~/.cache/ytmusic/), LRU eviction, lookup/register
storage.rs     → JSON persistence for favorites (HashSet<String>), playlists (Vec<Playlist>), queue (SavedQueue), history (Vec<HistoryEntry>)
ui/mod.rs      → ratatui immediate-mode rendering: library, search, queue, lyrics pane, home, favorites, playlists, history, explore, settings, player bar, help overlay, layout areas for mouse
ui/theme.rs    → style helpers (title, selected, accent, dim, etc.) using Theme struct
```

**Key design decisions:**
- mpv is controlled via IPC (not libmpv C bindings) — avoids C dependency, simpler cross-compilation
- Search runs in a background tokio task, results sent back via AppEvent channel — UI never blocks
- PlayerSender is Clone so commands can be sent without owning MpvProcess
- pending_load pattern: track-end handler sets a URL, tick() sends it to mpv — avoids async in sync event handlers
- Lyrics fetched from LRCLIB (free, no auth) via background tokio task; synced LRC timestamps parsed for real-time scrolling
- Config is TOML at `~/.config/ytmusic/config.toml`; theme/volume also changeable in-app via Settings screen
- All user data (favorites, playlists, queue, history) persisted as JSON in `~/.config/ytmusic/`
- Keybindings are data-driven: HashMap<Action, Vec<KeyBinding>> with TOML overrides merged on top of defaults
- History persists play timestamps; capped at 500 entries; used for both History view and Home page "Recently Played"
- Gapless playback via mpv's `loadfile append` — prefetches next track when within 10s of end
- Offline cache downloads tracks via background `yt-dlp` after playback starts; LRU eviction keeps disk bounded
- Mouse support via crossterm EnableMouseCapture; LayoutAreas stored for hit-testing clicks/scrolls
- Explore view fetches InnerTube `/browse` endpoint; supports drill-down navigation with breadcrumb stack

## Distribution

- **GitHub Actions CI:** lint (clippy + fmt) + build on ubuntu/macos/windows on every push
- **GitHub Actions Release:** on `git tag v*`, builds 5 binaries (linux x86/arm64, macos x86/arm64, windows x86), creates GH Release with checksums
- **install.sh:** curl one-liner that detects OS/arch, downloads binary, auto-installs mpv+yt-dlp via detected package manager
- **Homebrew formula:** `Formula/ytmusic.rb` with `depends_on "mpv"` and `depends_on "yt-dlp"`
- **crates.io:** package name is `ytmusic-tui`, binary name is `ytmusic`

## Dependencies (runtime)

Users need `mpv` and `yt-dlp` installed. The app checks at startup and prints install instructions if missing.

## Competitors

See `.context/competitor-analysis.md` for detailed comparison with ytermusic, youtui, ytui-music, and youtui-player.

**Closed gaps:** config file, theming (12 presets), custom keybindings, in-app settings, favorites, playlists, persistent queue, lyrics (synced via LRCLIB), home page, play history, explore/browse, offline caching, gapless playback, mouse support.

**Remaining gaps:** album art (terminal compatibility risk), media keys (souvlaki crate, behind feature flag).

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
