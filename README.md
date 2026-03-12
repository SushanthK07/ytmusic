# ytmusic

A keyboard-driven terminal UI client for YouTube Music, built in Rust.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
[![CI](https://github.com/SushanthK07/ytmusic/actions/workflows/ci.yml/badge.svg)](https://github.com/SushanthK07/ytmusic/actions/workflows/ci.yml)

```
╭─ ytmusic ─────────────────────────────────────────────────────╮
│ ╭─ Library ────────╮ ╭─ Search ─────────────────────────────╮ │
│ │                  │ │  Search: radiohead                   │ │
│ │  > Home          │ │                                      │ │
│ │    Search        │ │  > Creep          Radiohead    3:58 ♫│ │
│ │    Queue         │ │    Karma Police   Radiohead    4:22  │ │
│ │                  │ │    No Surprises   Radiohead    3:49  │ │
│ ├─ Queue (2) ──────┤ │    Everything..   Radiohead    4:33  │ │
│ │  1. Paranoid..   │ │                                      │ │
│ │  2. Fake Pla..   │ │                                      │ │
│ ╰──────────────────╯ ╰──────────────────────────────────────╯ │
│  ▶ Creep — Radiohead                           1:23 / 3:58   │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  space:pause  n/p:next/prev  /:search  ?:help  q:quit        │
╰───────────────────────────────────────────────────────────────╯
```

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

### Queue & Search

| Key | Action |
|-----|--------|
| `/` | Open search |
| `a` | Add selected track to queue |
| `d` / `x` | Remove from queue |
| `Esc` | Cancel search input |
| `Ctrl+u` | Clear search input |
| `Ctrl+w` | Delete word in search |

### General

| Key | Action |
|-----|--------|
| `?` | Toggle help overlay |
| `q` | Quit |

## Architecture

```
src/
├── main.rs        Entry point, terminal setup, dependency checks
├── app.rs         Application state, event loop, business logic
├── api.rs         YouTube Music InnerTube API client
├── player.rs      mpv IPC (JSON over Unix socket)
├── input.rs       Keyboard input handling (Normal / Search modes)
└── ui/
    ├── mod.rs     Layout and widget rendering
    └── theme.rs   Color palette
```

- **Event loop** — `tokio::select!` multiplexes terminal input, player events from mpv, API responses, and a tick timer
- **Search** — Hits YouTube Music's InnerTube API in a background tokio task; results arrive without blocking the UI
- **Playback** — Controls mpv via JSON IPC over a Unix socket; mpv handles yt-dlp integration internally

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
