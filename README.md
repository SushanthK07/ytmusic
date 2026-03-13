# ytmusic

A keyboard-driven terminal UI client for YouTube Music, built in Rust.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
[![CI](https://github.com/SushanthK07/ytmusic/actions/workflows/ci.yml/badge.svg)](https://github.com/SushanthK07/ytmusic/actions/workflows/ci.yml)

```
╭─ ytmusic ─────────────────────────────────────────────────────────────╮
│ ╭─ Library ────────╮ ╭─ Search ─────────────────────╮ ╭─ Lyrics ───╮ │
│ │  > Home          │ │  Search: radiohead            │ │            │ │
│ │    Search        │ │                               │ │ But I'm a  │ │
│ │    Explore       │ │  > Creep       Radiohead 3:58 │ │ creep      │ │
│ │    History       │ │    Karma..     Radiohead 4:22 │ │ I'm a      │ │
│ │    Favorites     │ │    No Sur..    Radiohead 3:49 │ │ weirdo     │ │
│ │    Playlists     │ │    Every..     Radiohead 4:33 │ │            │ │
│ │    Queue         │ │                               │ │            │ │
│ │    Settings      │ │                               │ │            │ │
│ ╰──────────────────╯ ╰──────────────────────────────╯ ╰────────────╯ │
│  ▶ Creep — Radiohead                                  1:23 / 3:58   │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  space:play/pause  n/p:next/prev  /:search  f:fav  L:lyrics  q:quit │
╰───────────────────────────────────────────────────────────────────────╯
```

## Features

- **Search & play** — search YouTube Music's catalog and stream audio via mpv
- **Synced lyrics** — real-time lyrics pane with auto-scrolling, powered by LRCLIB
- **Favorites** — like/unlike tracks, persisted locally
- **Playlists** — create, browse, and manage local playlists; add tracks from anywhere via picker overlay
- **Persistent queue** — queue and now-playing state saved across sessions
- **Play history** — tracks played are persisted with timestamps; browse and replay from History view
- **Explore** — browse YouTube Music's curated content (trending, charts, moods, genres) with drill-down navigation
- **Offline caching** — played tracks are background-downloaded and served from cache on replay; LRU eviction keeps disk usage bounded
- **Gapless playback** — next track is pre-loaded into mpv near the end of the current track, eliminating gaps
- **Mouse support** — click to select tracks, click progress bar to seek, scroll wheel to navigate lists
- **12 theme presets** — tokyo-night, dracula, gruvbox, nord, rose-pine, kanagawa, everforest, one-dark, solarized, mocha, latte, and default
- **Custom keybindings** — remap any action via TOML config
- **In-app settings** — change theme and volume without leaving the TUI
- **Vim-style navigation** — j/k, g/G, h/l, and all the keys you'd expect

## Installation

### One-liner (macOS / Linux) — recommended

```sh
curl -fsSL https://raw.githubusercontent.com/SushanthK07/ytmusic/main/install.sh | bash
```

This downloads the latest binary **and** installs dependencies (`mpv`, `yt-dlp`) automatically.

### Homebrew (macOS / Linux)

```sh
brew tap SushanthK07/ytmusic
brew install ytmusic
```

Homebrew handles `mpv` and `yt-dlp` as declared dependencies — nothing else to install.

### Cargo (any OS with Rust)

```sh
cargo install ytmusic-tui
```

> You still need `mpv` and `yt-dlp` installed separately (see [Dependencies](#dependencies) below).

### Manual download

Grab a pre-built binary from [Releases](https://github.com/SushanthK07/ytmusic/releases) for your platform, make it executable, and move it to your `$PATH`:

```sh
chmod +x ytmusic-*
sudo mv ytmusic-* /usr/local/bin/ytmusic
```

## Dependencies

ytmusic needs two runtime dependencies. The install script and Homebrew formula handle these automatically. If you installed manually:

| Dependency | Purpose | Install |
|------------|---------|---------|
| [mpv](https://mpv.io) | Audio playback | see below |
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | Stream extraction | see below |

<details>
<summary><strong>macOS</strong></summary>

```sh
brew install mpv yt-dlp
```
</details>

<details>
<summary><strong>Ubuntu / Debian</strong></summary>

```sh
sudo apt install mpv
pip install yt-dlp
```
</details>

<details>
<summary><strong>Arch Linux</strong></summary>

```sh
sudo pacman -S mpv yt-dlp
```
</details>

<details>
<summary><strong>Fedora</strong></summary>

```sh
sudo dnf install mpv
pip install yt-dlp
```
</details>

<details>
<summary><strong>Windows</strong></summary>

```sh
scoop install mpv yt-dlp
# or
choco install mpv yt-dlp
```
</details>

## Usage

```sh
ytmusic
```

Press `/` to search, `Enter` to play, `?` for help. That's it.

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Move cursor up / down |
| `g` / `G` | Jump to top / bottom |
| `Tab` / `h` / `l` | Switch panels |
| `Enter` | Select / play |

### Playback

| Key | Action |
|-----|--------|
| `Space` | Play / pause |
| `n` / `p` | Next / previous track |
| `>` / `<` (or `.` / `,`) | Seek forward / backward 5s |
| `+` / `-` | Volume up / down |
| `s` | Toggle shuffle |
| `r` | Cycle repeat (off → all → one) |

### Queue & Library

| Key | Action |
|-----|--------|
| `/` | Open search |
| `a` | Add selected track to queue |
| `A` | Play next (insert at front of queue) |
| `d` / `x` | Remove from queue |
| `f` | Toggle favorite |
| `P` | Add track to playlist |
| `L` | Toggle lyrics pane |
| `Esc` | Cancel search / go back |
| `Ctrl+u` | Clear search input |
| `Ctrl+w` | Delete word in search |

### General

| Key | Action |
|-----|--------|
| `?` | Toggle help overlay |
| `q` | Quit |

All keybindings can be customized via config file (see [Configuration](#configuration)).

## Configuration

ytmusic uses a TOML config file at `~/.config/ytmusic/config.toml` (auto-created on first run).

```toml
[general]
volume = 50
gapless = true          # pre-load next track for seamless transitions
cache_enabled = true    # background-download played tracks for offline replay
cache_max_size_mb = 2048

[theme]
# Presets: "default", "tokyo-night", "dracula", "gruvbox", "nord",
#          "rose-pine", "kanagawa", "everforest", "one-dark",
#          "solarized", "mocha", "latte"
preset = "tokyo-night"

# Override individual colors with hex values:
# accent = "#ff0000"
# border_active = "#ff0000"

[keybindings]
# Override any action: action = "key1, key2"
# Modifiers: "ctrl+c", "shift+a"
# quit = "q, ctrl+c"
# toggle_pause = "space"
```

Theme and volume can also be changed in-app via the Settings screen.

### Data files

All user data is stored in `~/.config/ytmusic/`:

| File | Contents |
|------|----------|
| `config.toml` | Theme, volume, keybindings, gapless, cache settings |
| `favorites.json` | Favorited track IDs |
| `playlists.json` | Saved playlists with tracks |
| `queue.json` | Queue and now-playing state |
| `history.json` | Play history (last 500 tracks with timestamps) |

Audio cache is stored at `~/.cache/ytmusic/` (macOS: `~/Library/Caches/ytmusic/`).

## Architecture

```
src/
├── main.rs        Entry point, terminal setup, mouse capture, dependency checks
├── app.rs         Application state, event loop, business logic
├── api.rs         YouTube Music InnerTube API + LRCLIB lyrics + browse/explore
├── player.rs      mpv IPC (JSON over Unix socket / named pipe)
├── input.rs       Keyboard + mouse input handling (Normal / Search / Explore modes)
├── config.rs      TOML config, theme presets, keybinding system
├── cache.rs       Audio cache with LRU eviction (background yt-dlp downloads)
├── storage.rs     JSON persistence (favorites, playlists, queue, history)
└── ui/
    ├── mod.rs     Layout and widget rendering (history, explore, mouse areas)
    └── theme.rs   Style helpers
```

- **Event loop** — `tokio::select!` multiplexes terminal input (keyboard + mouse), player events from mpv, API responses, and a tick timer
- **Search** — Hits YouTube Music's InnerTube API in a background tokio task; results arrive without blocking the UI
- **Playback** — Controls mpv via JSON IPC over a Unix socket (macOS/Linux) or named pipe (Windows); mpv handles yt-dlp integration internally
- **Lyrics** — Fetched from LRCLIB (free, no auth); supports synced (LRC) and plain text; displayed as a real-time scrolling pane
- **Caching** — Background `yt-dlp` download after playback starts; LRU eviction when cache exceeds configured max size
- **Gapless** — Pre-loads next track via mpv's `loadfile append` when within 10s of track end

## Building from source

```sh
git clone https://github.com/SushanthK07/ytmusic.git
cd ytmusic
cargo build --release
./target/release/ytmusic
```

## Releasing a new version

1. Bump version in `Cargo.toml`
2. Tag and push:
   ```sh
   git tag v0.1.0
   git push origin main --tags
   ```
3. GitHub Actions builds binaries for all platforms and creates a release automatically
4. Update the `sha256` values in `Formula/ytmusic.rb` from the release checksums

## License

MIT
