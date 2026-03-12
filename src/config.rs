use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ConfigFile {
    pub general: GeneralConfig,
    pub theme: ThemeConfig,
    pub keybindings: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub volume: i32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { volume: 50 }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub preset: String,
    pub accent: Option<String>,
    pub accent_dim: Option<String>,
    pub text: Option<String>,
    pub text_dim: Option<String>,
    pub text_secondary: Option<String>,
    pub border: Option<String>,
    pub border_active: Option<String>,
    pub highlight_bg: Option<String>,
    pub playing_indicator: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            preset: "default".to_string(),
            accent: None,
            accent_dim: None,
            text: None,
            text_dim: None,
            text_secondary: None,
            border: None,
            border_active: None,
            highlight_bg: None,
            playing_indicator: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ytmusic")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> Result<ConfigFile> {
    let path = config_path();
    if !path.exists() {
        create_default_config(&path)?;
        return Ok(ConfigFile::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: ConfigFile = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_theme_preset(preset: &str) -> Result<()> {
    let path = config_path();
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        DEFAULT_CONFIG.to_string()
    };

    let mut doc: toml::Table = toml::from_str(&content).unwrap_or_default();
    let theme_table = doc
        .entry("theme")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut();
    if let Some(t) = theme_table {
        t.insert(
            "preset".to_string(),
            toml::Value::String(preset.to_string()),
        );
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = toml::to_string_pretty(&doc)?;
    std::fs::write(&path, serialized)?;
    Ok(())
}

pub fn save_volume(volume: i32) -> Result<()> {
    let path = config_path();
    let content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        DEFAULT_CONFIG.to_string()
    };

    let mut doc: toml::Table = toml::from_str(&content).unwrap_or_default();
    let general_table = doc
        .entry("general")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut();
    if let Some(g) = general_table {
        g.insert("volume".to_string(), toml::Value::Integer(volume as i64));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = toml::to_string_pretty(&doc)?;
    std::fs::write(&path, serialized)?;
    Ok(())
}

fn create_default_config(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, DEFAULT_CONFIG)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Color,
    pub accent_dim: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub border_active: Color,
    pub highlight_bg: Color,
    pub playing_indicator: Color,
}

pub const THEME_PRESETS: &[&str] = &[
    "default",
    "tokyo-night",
    "dracula",
    "gruvbox",
    "nord",
    "rose-pine",
    "kanagawa",
    "everforest",
    "one-dark",
    "solarized",
    "mocha",
    "latte",
];

impl Theme {
    pub fn from_preset(name: &str) -> Self {
        match name {
            "tokyo-night" => Self::tokyo_night(),
            "dracula" => Self::dracula(),
            "gruvbox" => Self::gruvbox(),
            "nord" => Self::nord(),
            "rose-pine" => Self::rose_pine(),
            "kanagawa" => Self::kanagawa(),
            "everforest" => Self::everforest(),
            "one-dark" => Self::one_dark(),
            "solarized" => Self::solarized(),
            "mocha" => Self::catppuccin_mocha(),
            "latte" => Self::catppuccin_latte(),
            _ => Self::default(),
        }
    }

    pub fn from_config(config: &ThemeConfig) -> Self {
        let base = Self::from_preset(&config.preset);
        Self {
            accent: parse_hex_or(&config.accent, base.accent),
            accent_dim: parse_hex_or(&config.accent_dim, base.accent_dim),
            text: parse_hex_or(&config.text, base.text),
            text_dim: parse_hex_or(&config.text_dim, base.text_dim),
            text_secondary: parse_hex_or(&config.text_secondary, base.text_secondary),
            border: parse_hex_or(&config.border, base.border),
            border_active: parse_hex_or(&config.border_active, base.border_active),
            highlight_bg: parse_hex_or(&config.highlight_bg, base.highlight_bg),
            playing_indicator: parse_hex_or(&config.playing_indicator, base.playing_indicator),
        }
    }

    fn tokyo_night() -> Self {
        Self {
            accent: Color::Rgb(122, 162, 247),
            accent_dim: Color::Rgb(61, 89, 161),
            text: Color::Rgb(192, 202, 245),
            text_dim: Color::Rgb(86, 95, 137),
            text_secondary: Color::Rgb(115, 122, 162),
            border: Color::Rgb(59, 66, 97),
            border_active: Color::Rgb(122, 162, 247),
            highlight_bg: Color::Rgb(31, 35, 53),
            playing_indicator: Color::Rgb(158, 206, 106),
        }
    }

    fn dracula() -> Self {
        Self {
            accent: Color::Rgb(189, 147, 249),
            accent_dim: Color::Rgb(98, 114, 164),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(98, 114, 164),
            text_secondary: Color::Rgb(191, 191, 191),
            border: Color::Rgb(68, 71, 90),
            border_active: Color::Rgb(189, 147, 249),
            highlight_bg: Color::Rgb(68, 71, 90),
            playing_indicator: Color::Rgb(80, 250, 123),
        }
    }

    fn gruvbox() -> Self {
        Self {
            accent: Color::Rgb(254, 128, 25),
            accent_dim: Color::Rgb(214, 93, 14),
            text: Color::Rgb(235, 219, 178),
            text_dim: Color::Rgb(146, 131, 116),
            text_secondary: Color::Rgb(168, 153, 132),
            border: Color::Rgb(80, 73, 69),
            border_active: Color::Rgb(254, 128, 25),
            highlight_bg: Color::Rgb(60, 56, 54),
            playing_indicator: Color::Rgb(184, 187, 38),
        }
    }

    fn nord() -> Self {
        Self {
            accent: Color::Rgb(136, 192, 208),
            accent_dim: Color::Rgb(94, 129, 172),
            text: Color::Rgb(236, 239, 244),
            text_dim: Color::Rgb(76, 86, 106),
            text_secondary: Color::Rgb(216, 222, 233),
            border: Color::Rgb(59, 66, 82),
            border_active: Color::Rgb(136, 192, 208),
            highlight_bg: Color::Rgb(59, 66, 82),
            playing_indicator: Color::Rgb(163, 190, 140),
        }
    }

    fn rose_pine() -> Self {
        Self {
            accent: Color::Rgb(196, 167, 231),
            accent_dim: Color::Rgb(49, 116, 143),
            text: Color::Rgb(224, 222, 244),
            text_dim: Color::Rgb(110, 106, 134),
            text_secondary: Color::Rgb(144, 140, 170),
            border: Color::Rgb(64, 61, 82),
            border_active: Color::Rgb(196, 167, 231),
            highlight_bg: Color::Rgb(33, 32, 46),
            playing_indicator: Color::Rgb(156, 207, 216),
        }
    }

    fn kanagawa() -> Self {
        Self {
            accent: Color::Rgb(126, 156, 216),
            accent_dim: Color::Rgb(149, 127, 184),
            text: Color::Rgb(220, 215, 186),
            text_dim: Color::Rgb(114, 113, 105),
            text_secondary: Color::Rgb(200, 192, 147),
            border: Color::Rgb(54, 54, 70),
            border_active: Color::Rgb(126, 156, 216),
            highlight_bg: Color::Rgb(42, 42, 55),
            playing_indicator: Color::Rgb(152, 187, 108),
        }
    }

    fn everforest() -> Self {
        Self {
            accent: Color::Rgb(167, 192, 128),
            accent_dim: Color::Rgb(131, 192, 146),
            text: Color::Rgb(211, 198, 170),
            text_dim: Color::Rgb(122, 132, 120),
            text_secondary: Color::Rgb(157, 169, 160),
            border: Color::Rgb(71, 82, 88),
            border_active: Color::Rgb(167, 192, 128),
            highlight_bg: Color::Rgb(52, 63, 68),
            playing_indicator: Color::Rgb(131, 192, 146),
        }
    }

    fn one_dark() -> Self {
        Self {
            accent: Color::Rgb(97, 175, 239),
            accent_dim: Color::Rgb(82, 139, 255),
            text: Color::Rgb(171, 178, 191),
            text_dim: Color::Rgb(92, 99, 112),
            text_secondary: Color::Rgb(75, 82, 99),
            border: Color::Rgb(62, 68, 82),
            border_active: Color::Rgb(97, 175, 239),
            highlight_bg: Color::Rgb(44, 50, 60),
            playing_indicator: Color::Rgb(152, 195, 121),
        }
    }

    fn solarized() -> Self {
        Self {
            accent: Color::Rgb(38, 139, 210),
            accent_dim: Color::Rgb(42, 161, 152),
            text: Color::Rgb(131, 148, 150),
            text_dim: Color::Rgb(88, 110, 117),
            text_secondary: Color::Rgb(147, 161, 161),
            border: Color::Rgb(7, 54, 66),
            border_active: Color::Rgb(38, 139, 210),
            highlight_bg: Color::Rgb(7, 54, 66),
            playing_indicator: Color::Rgb(133, 153, 0),
        }
    }

    fn catppuccin_mocha() -> Self {
        Self {
            accent: Color::Rgb(203, 166, 247),
            accent_dim: Color::Rgb(137, 180, 250),
            text: Color::Rgb(205, 214, 244),
            text_dim: Color::Rgb(108, 112, 134),
            text_secondary: Color::Rgb(147, 153, 178),
            border: Color::Rgb(69, 71, 90),
            border_active: Color::Rgb(203, 166, 247),
            highlight_bg: Color::Rgb(49, 50, 68),
            playing_indicator: Color::Rgb(166, 227, 161),
        }
    }

    fn catppuccin_latte() -> Self {
        Self {
            accent: Color::Rgb(136, 57, 239),
            accent_dim: Color::Rgb(30, 102, 245),
            text: Color::Rgb(76, 79, 105),
            text_dim: Color::Rgb(156, 160, 176),
            text_secondary: Color::Rgb(124, 127, 147),
            border: Color::Rgb(188, 192, 204),
            border_active: Color::Rgb(136, 57, 239),
            highlight_bg: Color::Rgb(204, 208, 218),
            playing_indicator: Color::Rgb(64, 160, 43),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Red,
            accent_dim: Color::Indexed(88),
            text: Color::White,
            text_dim: Color::DarkGray,
            text_secondary: Color::Gray,
            border: Color::Indexed(236),
            border_active: Color::Red,
            highlight_bg: Color::Indexed(235),
            playing_indicator: Color::Green,
        }
    }
}

fn parse_hex_or(hex: &Option<String>, fallback: Color) -> Color {
    match hex {
        Some(s) => parse_hex_color(s).unwrap_or(fallback),
        None => fallback,
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    Search,
    Help,
    MoveDown,
    MoveUp,
    MoveTop,
    MoveBottom,
    NextPanel,
    PrevPanel,
    Select,
    TogglePause,
    NextTrack,
    PrevTrack,
    SeekForward,
    SeekBackward,
    VolumeUp,
    VolumeDown,
    ToggleShuffle,
    ToggleRepeat,
    AddToQueue,
    PlayNext,
    RemoveFromQueue,
}

impl Action {
    fn from_name(s: &str) -> Option<Self> {
        match s {
            "quit" => Some(Self::Quit),
            "search" => Some(Self::Search),
            "help" => Some(Self::Help),
            "move_down" => Some(Self::MoveDown),
            "move_up" => Some(Self::MoveUp),
            "move_top" => Some(Self::MoveTop),
            "move_bottom" => Some(Self::MoveBottom),
            "next_panel" => Some(Self::NextPanel),
            "prev_panel" => Some(Self::PrevPanel),
            "select" => Some(Self::Select),
            "toggle_pause" => Some(Self::TogglePause),
            "next_track" => Some(Self::NextTrack),
            "prev_track" => Some(Self::PrevTrack),
            "seek_forward" => Some(Self::SeekForward),
            "seek_backward" => Some(Self::SeekBackward),
            "volume_up" => Some(Self::VolumeUp),
            "volume_down" => Some(Self::VolumeDown),
            "toggle_shuffle" => Some(Self::ToggleShuffle),
            "toggle_repeat" => Some(Self::ToggleRepeat),
            "add_to_queue" => Some(Self::AddToQueue),
            "play_next" => Some(Self::PlayNext),
            "remove_from_queue" => Some(Self::RemoveFromQueue),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub map: HashMap<Action, Vec<KeyBinding>>,
}

impl KeyBindings {
    pub fn from_config(overrides: &HashMap<String, String>) -> Self {
        let mut bindings = Self::defaults();

        for (action_name, keys_str) in overrides {
            if let Some(action) = Action::from_name(action_name) {
                let parsed: Vec<KeyBinding> = keys_str
                    .split(',')
                    .filter_map(|k| parse_key_binding(k.trim()))
                    .collect();
                if !parsed.is_empty() {
                    bindings.map.insert(action, parsed);
                }
            }
        }

        bindings
    }

    pub fn matches(&self, action: Action, code: &KeyCode, modifiers: &KeyModifiers) -> bool {
        if let Some(bindings) = self.map.get(&action) {
            bindings
                .iter()
                .any(|b| b.code == *code && modifiers.contains(b.modifiers))
        } else {
            false
        }
    }

    fn defaults() -> Self {
        let mut map = HashMap::new();

        let bind = |code: KeyCode| KeyBinding {
            code,
            modifiers: KeyModifiers::NONE,
        };
        let bind_shift = |code: KeyCode| KeyBinding {
            code,
            modifiers: KeyModifiers::SHIFT,
        };
        let bind_ctrl = |code: KeyCode| KeyBinding {
            code,
            modifiers: KeyModifiers::CONTROL,
        };

        map.insert(
            Action::Quit,
            vec![bind(KeyCode::Char('q')), bind_ctrl(KeyCode::Char('c'))],
        );
        map.insert(Action::Search, vec![bind(KeyCode::Char('/'))]);
        map.insert(Action::Help, vec![bind(KeyCode::Char('?'))]);
        map.insert(
            Action::MoveDown,
            vec![bind(KeyCode::Char('j')), bind(KeyCode::Down)],
        );
        map.insert(
            Action::MoveUp,
            vec![bind(KeyCode::Char('k')), bind(KeyCode::Up)],
        );
        map.insert(Action::MoveTop, vec![bind(KeyCode::Char('g'))]);
        map.insert(Action::MoveBottom, vec![bind_shift(KeyCode::Char('G'))]);
        map.insert(
            Action::NextPanel,
            vec![
                bind(KeyCode::Tab),
                bind(KeyCode::Char('l')),
                bind(KeyCode::Right),
            ],
        );
        map.insert(
            Action::PrevPanel,
            vec![
                bind(KeyCode::BackTab),
                bind(KeyCode::Char('h')),
                bind(KeyCode::Left),
            ],
        );
        map.insert(Action::Select, vec![bind(KeyCode::Enter)]);
        map.insert(Action::TogglePause, vec![bind(KeyCode::Char(' '))]);
        map.insert(Action::NextTrack, vec![bind(KeyCode::Char('n'))]);
        map.insert(Action::PrevTrack, vec![bind(KeyCode::Char('p'))]);
        map.insert(
            Action::SeekForward,
            vec![bind_shift(KeyCode::Char('>')), bind(KeyCode::Char('.'))],
        );
        map.insert(
            Action::SeekBackward,
            vec![bind_shift(KeyCode::Char('<')), bind(KeyCode::Char(','))],
        );
        map.insert(
            Action::VolumeUp,
            vec![bind_shift(KeyCode::Char('+')), bind(KeyCode::Char('='))],
        );
        map.insert(
            Action::VolumeDown,
            vec![bind(KeyCode::Char('-')), bind_shift(KeyCode::Char('_'))],
        );
        map.insert(Action::ToggleShuffle, vec![bind(KeyCode::Char('s'))]);
        map.insert(Action::ToggleRepeat, vec![bind(KeyCode::Char('r'))]);
        map.insert(Action::AddToQueue, vec![bind(KeyCode::Char('a'))]);
        map.insert(Action::PlayNext, vec![bind_shift(KeyCode::Char('A'))]);
        map.insert(
            Action::RemoveFromQueue,
            vec![bind(KeyCode::Char('d')), bind(KeyCode::Char('x'))],
        );

        Self { map }
    }
}

fn parse_key_binding(s: &str) -> Option<KeyBinding> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = KeyModifiers::NONE;

    for &part in &parts[..parts.len().saturating_sub(1)] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" => modifiers |= KeyModifiers::ALT,
            _ => return None,
        }
    }

    let key_str = parts.last()?;
    let code = match key_str.to_lowercase().as_str() {
        "space" => KeyCode::Char(' '),
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        s if s.len() == 1 => {
            let c = s.chars().next()?;
            if modifiers.contains(KeyModifiers::SHIFT) && c.is_ascii_alphabetic() {
                KeyCode::Char(c.to_ascii_uppercase())
            } else {
                KeyCode::Char(c)
            }
        }
        _ => return None,
    };

    Some(KeyBinding { code, modifiers })
}

const DEFAULT_CONFIG: &str = r##"# ytmusic configuration
# Location: ~/.config/ytmusic/config.toml

[general]
volume = 50

[theme]
# Preset: "default", "tokyo-night", "dracula", "gruvbox", "nord",
#         "rose-pine", "kanagawa", "everforest", "one-dark",
#         "solarized", "mocha", "latte"
preset = "default"

# Override individual colors with hex values:
# accent = "#ff0000"
# text = "#ffffff"
# text_dim = "#555555"
# text_secondary = "#888888"
# border = "#333333"
# border_active = "#ff0000"
# highlight_bg = "#1a1a1a"
# playing_indicator = "#00ff00"

[keybindings]
# Override keybindings with action = "key" pairs.
# Multiple keys: "key1, key2"
# Modifiers: "ctrl+c", "shift+a"
# Special keys: space, enter, esc, tab, up, down, left, right
#
# Available actions:
#   quit, search, help,
#   move_down, move_up, move_top, move_bottom,
#   next_panel, prev_panel, select,
#   toggle_pause, next_track, prev_track,
#   seek_forward, seek_backward,
#   volume_up, volume_down,
#   toggle_shuffle, toggle_repeat,
#   add_to_queue, play_next, remove_from_queue
#
# Examples:
# quit = "q, ctrl+c"
# toggle_pause = "space"
# next_track = "n"
"##;
