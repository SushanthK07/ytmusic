use crate::api::{self, LyricLine, LyricsResponse, Track, YtMusicClient};
use crate::config::{self, KeyBindings, Theme, THEME_PRESETS};
use crate::player::{
    MpvProcess, PlaybackState, PlayerCommand, PlayerEvent, PlayerSender, PlayerStatus,
};
use crate::storage::{self, Playlist};
use anyhow::Result;
use std::collections::HashSet;
use tokio::sync::mpsc;

pub enum AppEvent {
    Player(PlayerEvent),
    SearchResults(Result<Vec<Track>, String>),
    LyricsResult(String, Option<LyricsResponse>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Library,
    Content,
    Queue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryItem {
    Home,
    Search,
    Favorites,
    Playlists,
    Queue,
    Settings,
}

impl LibraryItem {
    pub const ALL: [LibraryItem; 6] = [
        LibraryItem::Home,
        LibraryItem::Search,
        LibraryItem::Favorites,
        LibraryItem::Playlists,
        LibraryItem::Queue,
        LibraryItem::Settings,
    ];

    pub fn label(&self) -> &str {
        match self {
            LibraryItem::Home => "Home",
            LibraryItem::Search => "Search",
            LibraryItem::Favorites => "Favorites",
            LibraryItem::Playlists => "Playlists",
            LibraryItem::Queue => "Queue",
            LibraryItem::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaylistMode {
    List,
    View,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsSection {
    Theme,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn label(&self) -> &str {
        match self {
            RepeatMode::Off => "off",
            RepeatMode::All => "all",
            RepeatMode::One => "one",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub active_panel: Panel,
    pub should_quit: bool,
    pub show_help: bool,

    pub library_cursor: usize,

    pub search_input: String,
    pub search_cursor: usize,
    pub search_results: Vec<Track>,
    pub search_result_cursor: usize,
    pub is_searching: bool,

    pub queue: Vec<Track>,
    pub queue_cursor: usize,
    pub history: Vec<Track>,

    pub now_playing: Option<Track>,
    pub player_status: PlayerStatus,
    pub shuffle: bool,
    pub repeat: RepeatMode,

    pub notification: Option<(String, std::time::Instant)>,

    pub show_playlist_picker: bool,
    pub playlist_picker_cursor: usize,
    pub playlist_picker_track: Option<Track>,
    pub playlist_picker_creating: bool,

    pub favorites: HashSet<String>,
    pub favorites_tracks: Vec<Track>,
    pub favorites_cursor: usize,

    pub playlists: Vec<Playlist>,
    pub playlist_cursor: usize,
    pub playlist_mode: PlaylistMode,
    pub viewing_playlist: Option<usize>,
    pub playlist_track_cursor: usize,
    pub playlist_name_input: String,
    pub playlist_name_cursor: usize,

    pub show_lyrics: bool,
    pub lyrics_lines: Vec<String>,
    pub synced_lyrics: Vec<LyricLine>,
    pub lyrics_scroll: usize,
    pub lyrics_loading: bool,
    lyrics_video_id: Option<String>,

    pub theme: Theme,
    pub keybindings: KeyBindings,
    pub settings_section: SettingsSection,
    pub settings_cursor: usize,
    pub theme_cursor: usize,
    pub current_theme_name: String,

    pub api: YtMusicClient,
    player_sender: Option<PlayerSender>,
    _mpv: Option<MpvProcess>,
    event_rx: mpsc::Receiver<AppEvent>,
    event_tx: mpsc::Sender<AppEvent>,
    pending_load: Option<String>,
}

impl App {
    pub async fn new(
        initial_volume: i32,
        theme: Theme,
        keybindings: KeyBindings,
        theme_name: String,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>(256);

        let (player_tx, mut player_rx) = mpsc::channel::<PlayerEvent>(256);
        let bridge_tx = event_tx.clone();
        tokio::spawn(async move {
            while let Some(pe) = player_rx.recv().await {
                if bridge_tx.send(AppEvent::Player(pe)).await.is_err() {
                    break;
                }
            }
        });

        let (mpv, sender) = match MpvProcess::spawn(player_tx).await {
            Ok(proc) => {
                let sender = proc.sender.clone();
                (Some(proc), Some(sender))
            }
            Err(_) => (None, None),
        };

        let mut app = Self {
            mode: Mode::Normal,
            active_panel: Panel::Content,
            should_quit: false,
            show_help: false,

            library_cursor: 0,

            search_input: String::new(),
            search_cursor: 0,
            search_results: Vec::new(),
            search_result_cursor: 0,
            is_searching: false,

            queue: {
                let saved = storage::load_queue();
                saved.tracks
            },
            queue_cursor: 0,
            history: Vec::new(),

            now_playing: None,
            player_status: PlayerStatus {
                volume: initial_volume as i64,
                ..PlayerStatus::default()
            },
            shuffle: false,
            repeat: RepeatMode::Off,

            notification: None,

            show_playlist_picker: false,
            playlist_picker_cursor: 0,
            playlist_picker_track: None,
            playlist_picker_creating: false,

            favorites: storage::load_favorites(),
            favorites_tracks: Vec::new(),
            favorites_cursor: 0,

            playlists: storage::load_playlists(),
            playlist_cursor: 0,
            playlist_mode: PlaylistMode::List,
            viewing_playlist: None,
            playlist_track_cursor: 0,
            playlist_name_input: String::new(),
            playlist_name_cursor: 0,

            show_lyrics: false,
            lyrics_lines: Vec::new(),
            synced_lyrics: Vec::new(),
            lyrics_scroll: 0,
            lyrics_loading: false,
            lyrics_video_id: None,

            theme,
            keybindings,
            settings_section: SettingsSection::Theme,
            settings_cursor: 0,
            theme_cursor: THEME_PRESETS
                .iter()
                .position(|&p| p == theme_name)
                .unwrap_or(0),
            current_theme_name: theme_name,

            api: YtMusicClient::new(),
            player_sender: sender,
            _mpv: mpv,
            event_rx,
            event_tx,
            pending_load: None,
        };

        app.load_favorites_tracks();
        Ok(app)
    }

    pub async fn tick(&mut self) {
        self.drain_events();
        self.clear_stale_notification();

        if let Some(url) = self.pending_load.take() {
            if let Some(ref sender) = self.player_sender {
                let _ = sender.send(PlayerCommand::Load(url)).await;
            }
        }
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::Player(pe) => match pe {
                    PlayerEvent::Position(pos) => self.player_status.position = pos,
                    PlayerEvent::Duration(dur) => self.player_status.duration = dur,
                    PlayerEvent::Paused(paused) => {
                        self.player_status.state = if paused {
                            PlaybackState::Paused
                        } else {
                            PlaybackState::Playing
                        };
                    }
                    PlayerEvent::Idle => {
                        if self.player_status.state != PlaybackState::Stopped {
                            self.player_status.state = PlaybackState::Stopped;
                        }
                    }
                    PlayerEvent::TrackEnd => self.on_track_end(),
                    PlayerEvent::Error(msg) => self.notify(format!("Playback error: {}", msg)),
                },
                AppEvent::SearchResults(result) => {
                    self.is_searching = false;
                    match result {
                        Ok(tracks) => {
                            if tracks.is_empty() {
                                self.notify("No results found".to_string());
                            }
                            self.search_results = tracks;
                        }
                        Err(e) => self.notify(format!("Search failed: {}", e)),
                    }
                }
                AppEvent::LyricsResult(video_id, response) => {
                    if self
                        .now_playing
                        .as_ref()
                        .map(|t| t.video_id == video_id)
                        .unwrap_or(false)
                    {
                        self.lyrics_loading = false;
                        self.lyrics_scroll = 0;
                        match response {
                            Some(lr) if lr.instrumental == Some(true) => {
                                self.lyrics_lines = vec!["♫ Instrumental".to_string()];
                                self.synced_lyrics.clear();
                            }
                            Some(lr) => {
                                if let Some(ref synced) = lr.synced_lyrics {
                                    self.synced_lyrics = api::parse_synced_lyrics(synced);
                                    self.lyrics_lines =
                                        self.synced_lyrics.iter().map(|l| l.text.clone()).collect();
                                } else if let Some(ref plain) = lr.plain_lyrics {
                                    self.synced_lyrics.clear();
                                    self.lyrics_lines =
                                        plain.lines().map(|l| l.to_string()).collect();
                                } else {
                                    self.lyrics_lines = vec!["No lyrics available".to_string()];
                                    self.synced_lyrics.clear();
                                }
                            }
                            None => {
                                self.lyrics_lines = vec!["No lyrics available".to_string()];
                                self.synced_lyrics.clear();
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_track_end(&mut self) {
        match self.repeat {
            RepeatMode::One => {
                if let Some(track) = &self.now_playing {
                    self.pending_load = Some(track.youtube_url());
                }
            }
            _ => self.advance_queue(),
        }
    }

    fn advance_queue(&mut self) {
        if self.queue.is_empty() {
            if self.repeat == RepeatMode::All && !self.history.is_empty() {
                self.queue = self.history.clone();
            } else {
                self.player_status.state = PlaybackState::Stopped;
                self.now_playing = None;
                return;
            }
        }

        let track = if self.shuffle {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            let idx = (hasher.finish() as usize) % self.queue.len();
            self.queue.remove(idx)
        } else {
            self.queue.remove(0)
        };

        self.pending_load = Some(track.youtube_url());
        self.now_playing = Some(track.clone());
        self.player_status.state = PlaybackState::Buffering;
        self.player_status.position = 0.0;
        self.player_status.duration = 0.0;
        self.history.push(track);
    }

    fn clear_stale_notification(&mut self) {
        if let Some((_, created)) = &self.notification {
            if created.elapsed() > std::time::Duration::from_secs(3) {
                self.notification = None;
            }
        }
    }

    pub fn notify(&mut self, msg: String) {
        self.notification = Some((msg, std::time::Instant::now()));
    }

    pub fn enter_search(&mut self) {
        self.mode = Mode::Search;
        self.active_panel = Panel::Content;
        self.library_cursor = 1;
    }

    pub fn exit_search(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn submit_search(&mut self) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            return;
        }

        self.is_searching = true;
        self.mode = Mode::Normal;
        self.search_result_cursor = 0;

        let api = self.api.clone();
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            let result = api.search(&query).await.map_err(|e| e.to_string());
            let _ = tx.send(AppEvent::SearchResults(result)).await;
        });
    }

    pub async fn play_track(&mut self, track: Track) {
        let url = track.youtube_url();
        self.fetch_lyrics(&track);
        self.now_playing = Some(track.clone());
        self.player_status.state = PlaybackState::Buffering;
        self.player_status.position = 0.0;
        self.player_status.duration = 0.0;
        self.history.push(track);

        if let Some(ref sender) = self.player_sender {
            if let Err(e) = sender.send(PlayerCommand::Load(url)).await {
                self.notify(format!("Failed to play: {}", e));
            }
        } else {
            self.notify("mpv not available — install mpv and yt-dlp".to_string());
        }
    }

    fn fetch_lyrics(&mut self, track: &Track) {
        if self.lyrics_video_id.as_deref() == Some(&track.video_id) {
            return;
        }
        self.lyrics_video_id = Some(track.video_id.clone());
        self.lyrics_loading = true;
        self.lyrics_lines.clear();
        self.synced_lyrics.clear();
        self.lyrics_scroll = 0;

        let api = self.api.clone();
        let tx = self.event_tx.clone();
        let title = track.title.clone();
        let artist = track.artist.clone();
        let video_id = track.video_id.clone();
        let duration = track
            .duration_text
            .as_deref()
            .and_then(api::duration_text_to_secs);

        tokio::spawn(async move {
            let result = api.fetch_lyrics(&title, &artist, duration).await;
            let response = result.ok().flatten();
            let _ = tx.send(AppEvent::LyricsResult(video_id, response)).await;
        });
    }

    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
    }

    pub fn current_lyric_index(&self) -> Option<usize> {
        if self.synced_lyrics.is_empty() {
            return None;
        }
        let pos_ms = (self.player_status.position * 1000.0) as u64;
        let mut idx = 0;
        for (i, line) in self.synced_lyrics.iter().enumerate() {
            if line.time_ms <= pos_ms {
                idx = i;
            } else {
                break;
            }
        }
        Some(idx)
    }

    pub async fn play_selected(&mut self) {
        match self.active_panel {
            Panel::Content if !self.search_results.is_empty() => {
                let track = self.search_results[self.search_result_cursor].clone();
                let remaining: Vec<Track> = self
                    .search_results
                    .iter()
                    .skip(self.search_result_cursor + 1)
                    .cloned()
                    .collect();
                self.queue = remaining;
                self.queue_cursor = 0;
                self.play_track(track).await;
            }
            Panel::Queue if !self.queue.is_empty() => {
                let track = self.queue.remove(self.queue_cursor);
                if self.queue_cursor >= self.queue.len() && !self.queue.is_empty() {
                    self.queue_cursor = self.queue.len() - 1;
                }
                self.play_track(track).await;
            }
            _ => {}
        }
    }

    pub fn add_to_queue(&mut self) {
        if self.active_panel == Panel::Content && !self.search_results.is_empty() {
            let track = self.search_results[self.search_result_cursor].clone();
            self.notify(format!("Queued: {}", track.title));
            self.queue.push(track);
        }
    }

    pub fn play_next_in_queue(&mut self) {
        if self.active_panel == Panel::Content && !self.search_results.is_empty() {
            let track = self.search_results[self.search_result_cursor].clone();
            self.notify(format!("Playing next: {}", track.title));
            self.queue.insert(0, track);
        }
    }

    pub async fn toggle_pause(&mut self) {
        if let Some(ref sender) = self.player_sender {
            let _ = sender.send(PlayerCommand::TogglePause).await;
        }
    }

    pub fn play_next(&mut self) {
        self.advance_queue();
    }

    pub fn play_prev(&mut self) {
        if self.history.len() < 2 {
            return;
        }

        if let Some(current) = self.now_playing.take() {
            self.queue.insert(0, current);
        }

        self.history.pop();
        if let Some(track) = self.history.last().cloned() {
            self.pending_load = Some(track.youtube_url());
            self.now_playing = Some(track);
            self.player_status.state = PlaybackState::Buffering;
            self.player_status.position = 0.0;
        }
    }

    pub async fn seek_forward(&mut self) {
        if let Some(ref sender) = self.player_sender {
            let _ = sender.send(PlayerCommand::SeekForward(5.0)).await;
        }
    }

    pub async fn seek_backward(&mut self) {
        if let Some(ref sender) = self.player_sender {
            let _ = sender.send(PlayerCommand::SeekBackward(5.0)).await;
        }
    }

    pub async fn volume_up(&mut self) {
        if let Some(ref sender) = self.player_sender {
            let _ = sender.send(PlayerCommand::VolumeUp).await;
            self.player_status.volume = (self.player_status.volume + 5).min(100);
        }
    }

    pub async fn volume_down(&mut self) {
        if let Some(ref sender) = self.player_sender {
            let _ = sender.send(PlayerCommand::VolumeDown).await;
            self.player_status.volume = (self.player_status.volume - 5).max(0);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.notify(format!(
            "Shuffle: {}",
            if self.shuffle { "on" } else { "off" }
        ));
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = self.repeat.cycle();
        self.notify(format!("Repeat: {}", self.repeat.label()));
    }

    pub fn move_cursor_up(&mut self) {
        match self.active_panel {
            Panel::Library => self.library_cursor = self.library_cursor.saturating_sub(1),
            Panel::Content => match self.selected_library_item() {
                LibraryItem::Favorites => {
                    self.favorites_cursor = self.favorites_cursor.saturating_sub(1)
                }
                LibraryItem::Playlists => match self.playlist_mode {
                    PlaylistMode::List => {
                        self.playlist_cursor = self.playlist_cursor.saturating_sub(1)
                    }
                    PlaylistMode::View => {
                        self.playlist_track_cursor = self.playlist_track_cursor.saturating_sub(1)
                    }
                    PlaylistMode::Create => {}
                },
                _ => self.search_result_cursor = self.search_result_cursor.saturating_sub(1),
            },
            Panel::Queue => self.queue_cursor = self.queue_cursor.saturating_sub(1),
        }
    }

    pub fn move_cursor_down(&mut self) {
        match self.active_panel {
            Panel::Library => {
                if self.library_cursor < LibraryItem::ALL.len() - 1 {
                    self.library_cursor += 1;
                }
            }
            Panel::Content => match self.selected_library_item() {
                LibraryItem::Favorites => {
                    if !self.favorites_tracks.is_empty()
                        && self.favorites_cursor < self.favorites_tracks.len() - 1
                    {
                        self.favorites_cursor += 1;
                    }
                }
                LibraryItem::Playlists => match self.playlist_mode {
                    PlaylistMode::List => {
                        if !self.playlists.is_empty()
                            && self.playlist_cursor < self.playlists.len() - 1
                        {
                            self.playlist_cursor += 1;
                        }
                    }
                    PlaylistMode::View => {
                        if let Some(idx) = self.viewing_playlist {
                            if let Some(pl) = self.playlists.get(idx) {
                                if !pl.tracks.is_empty()
                                    && self.playlist_track_cursor < pl.tracks.len() - 1
                                {
                                    self.playlist_track_cursor += 1;
                                }
                            }
                        }
                    }
                    PlaylistMode::Create => {}
                },
                _ => {
                    if !self.search_results.is_empty()
                        && self.search_result_cursor < self.search_results.len() - 1
                    {
                        self.search_result_cursor += 1;
                    }
                }
            },
            Panel::Queue => {
                if !self.queue.is_empty() && self.queue_cursor < self.queue.len() - 1 {
                    self.queue_cursor += 1;
                }
            }
        }
    }

    pub fn move_cursor_top(&mut self) {
        match self.active_panel {
            Panel::Library => self.library_cursor = 0,
            Panel::Content => match self.selected_library_item() {
                LibraryItem::Favorites => self.favorites_cursor = 0,
                LibraryItem::Playlists => match self.playlist_mode {
                    PlaylistMode::List => self.playlist_cursor = 0,
                    PlaylistMode::View => self.playlist_track_cursor = 0,
                    PlaylistMode::Create => {}
                },
                _ => self.search_result_cursor = 0,
            },
            Panel::Queue => self.queue_cursor = 0,
        }
    }

    pub fn move_cursor_bottom(&mut self) {
        match self.active_panel {
            Panel::Library => self.library_cursor = LibraryItem::ALL.len().saturating_sub(1),
            Panel::Content => match self.selected_library_item() {
                LibraryItem::Favorites => {
                    self.favorites_cursor = self.favorites_tracks.len().saturating_sub(1)
                }
                LibraryItem::Playlists => match self.playlist_mode {
                    PlaylistMode::List => {
                        self.playlist_cursor = self.playlists.len().saturating_sub(1)
                    }
                    PlaylistMode::View => {
                        if let Some(idx) = self.viewing_playlist {
                            if let Some(pl) = self.playlists.get(idx) {
                                self.playlist_track_cursor = pl.tracks.len().saturating_sub(1);
                            }
                        }
                    }
                    PlaylistMode::Create => {}
                },
                _ => self.search_result_cursor = self.search_results.len().saturating_sub(1),
            },
            Panel::Queue => self.queue_cursor = self.queue.len().saturating_sub(1),
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Library => Panel::Content,
            Panel::Content => Panel::Queue,
            Panel::Queue => Panel::Library,
        };
    }

    pub fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Library => Panel::Queue,
            Panel::Content => Panel::Library,
            Panel::Queue => Panel::Content,
        };
    }

    pub fn remove_from_queue(&mut self) {
        if self.active_panel == Panel::Queue && !self.queue.is_empty() {
            self.queue.remove(self.queue_cursor);
            if self.queue_cursor >= self.queue.len() && !self.queue.is_empty() {
                self.queue_cursor = self.queue.len() - 1;
            }
        }
    }

    pub fn selected_library_item(&self) -> LibraryItem {
        LibraryItem::ALL[self.library_cursor]
    }

    pub fn toggle_favorite(&mut self) {
        let track = match self.active_panel {
            Panel::Content if self.selected_library_item() == LibraryItem::Favorites => {
                self.favorites_tracks.get(self.favorites_cursor).cloned()
            }
            Panel::Content if !self.search_results.is_empty() => {
                Some(self.search_results[self.search_result_cursor].clone())
            }
            Panel::Queue if !self.queue.is_empty() => Some(self.queue[self.queue_cursor].clone()),
            _ => self.now_playing.clone(),
        };

        if let Some(track) = track {
            if self.favorites.contains(&track.video_id) {
                self.favorites.remove(&track.video_id);
                self.favorites_tracks
                    .retain(|t| t.video_id != track.video_id);
                if self.favorites_cursor >= self.favorites_tracks.len()
                    && !self.favorites_tracks.is_empty()
                {
                    self.favorites_cursor = self.favorites_tracks.len() - 1;
                }
                self.notify(format!("Unfavorited: {}", track.title));
            } else {
                self.favorites.insert(track.video_id.clone());
                self.favorites_tracks.push(track.clone());
                self.notify(format!("Favorited: {}", track.title));
            }
            storage::save_favorites(&self.favorites);
            self.save_favorites_tracks();
        }
    }

    pub fn is_favorited(&self, video_id: &str) -> bool {
        self.favorites.contains(video_id)
    }

    fn save_favorites_tracks(&self) {
        let fav_tracks: Vec<&Track> = self
            .favorites_tracks
            .iter()
            .filter(|t| self.favorites.contains(&t.video_id))
            .collect();
        let serialized = serde_json::to_string_pretty(&fav_tracks).unwrap_or_default();
        let path = crate::config::config_dir().join("favorites_tracks.json");
        let _ = std::fs::write(path, serialized);
    }

    pub fn load_favorites_tracks(&mut self) {
        let path = crate::config::config_dir().join("favorites_tracks.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(tracks) = serde_json::from_str::<Vec<Track>>(&content) {
                    self.favorites_tracks = tracks
                        .into_iter()
                        .filter(|t| self.favorites.contains(&t.video_id))
                        .collect();
                }
            }
        }
    }

    pub fn create_playlist(&mut self) {
        let name = self.playlist_name_input.trim().to_string();
        if name.is_empty() {
            return;
        }
        self.playlists.push(Playlist {
            name: name.clone(),
            tracks: Vec::new(),
        });
        storage::save_playlists(&self.playlists);
        self.playlist_name_input.clear();
        self.playlist_name_cursor = 0;
        self.playlist_mode = PlaylistMode::List;
        self.notify(format!("Created playlist: {}", name));
    }

    pub fn delete_playlist(&mut self) {
        if self.playlist_mode == PlaylistMode::List && !self.playlists.is_empty() {
            let name = self.playlists[self.playlist_cursor].name.clone();
            self.playlists.remove(self.playlist_cursor);
            if self.playlist_cursor >= self.playlists.len() && !self.playlists.is_empty() {
                self.playlist_cursor = self.playlists.len() - 1;
            }
            storage::save_playlists(&self.playlists);
            self.notify(format!("Deleted playlist: {}", name));
        }
    }

    pub fn open_playlist_picker(&mut self) {
        let track = match self.active_panel {
            Panel::Content
                if self.selected_library_item() == LibraryItem::Favorites
                    && !self.favorites_tracks.is_empty() =>
            {
                Some(self.favorites_tracks[self.favorites_cursor].clone())
            }
            Panel::Content if !self.search_results.is_empty() => {
                Some(self.search_results[self.search_result_cursor].clone())
            }
            Panel::Queue if !self.queue.is_empty() => Some(self.queue[self.queue_cursor].clone()),
            _ => self.now_playing.clone(),
        };

        match track {
            None => self.notify("No track selected".to_string()),
            Some(t) => {
                self.playlist_picker_track = Some(t);
                self.playlist_picker_cursor = 0;
                self.playlist_picker_creating = false;
                self.show_playlist_picker = true;
            }
        }
    }

    pub fn confirm_playlist_picker(&mut self) {
        if self.playlist_picker_cursor == self.playlists.len() {
            self.playlist_picker_creating = true;
            self.playlist_name_input.clear();
            self.playlist_name_cursor = 0;
            return;
        }
        if let Some(track) = self.playlist_picker_track.take() {
            let idx = self.playlist_picker_cursor;
            if idx < self.playlists.len() {
                let name = self.playlists[idx].name.clone();
                self.playlists[idx].tracks.push(track.clone());
                storage::save_playlists(&self.playlists);
                self.notify(format!("Added to {}: {}", name, track.title));
            }
        }
        self.show_playlist_picker = false;
    }

    pub fn picker_create_playlist(&mut self) {
        let name = self.playlist_name_input.trim().to_string();
        if name.is_empty() {
            return;
        }
        self.playlists.push(Playlist {
            name: name.clone(),
            tracks: Vec::new(),
        });
        let new_idx = self.playlists.len() - 1;
        if let Some(track) = self.playlist_picker_track.take() {
            self.playlists[new_idx].tracks.push(track.clone());
            self.notify(format!("Created '{}' and added: {}", name, track.title));
        }
        storage::save_playlists(&self.playlists);
        self.playlist_name_input.clear();
        self.playlist_name_cursor = 0;
        self.playlist_picker_creating = false;
        self.show_playlist_picker = false;
    }

    pub fn close_playlist_picker(&mut self) {
        self.show_playlist_picker = false;
        self.playlist_picker_track = None;
        self.playlist_picker_creating = false;
        self.playlist_name_input.clear();
        self.playlist_name_cursor = 0;
    }

    pub fn remove_from_playlist(&mut self) {
        if let Some(idx) = self.viewing_playlist {
            if let Some(playlist) = self.playlists.get_mut(idx) {
                if !playlist.tracks.is_empty() {
                    playlist.tracks.remove(self.playlist_track_cursor);
                    if self.playlist_track_cursor >= playlist.tracks.len()
                        && !playlist.tracks.is_empty()
                    {
                        self.playlist_track_cursor = playlist.tracks.len() - 1;
                    }
                    storage::save_playlists(&self.playlists);
                }
            }
        }
    }

    pub async fn play_favorites(&mut self) {
        if self.favorites_tracks.is_empty() {
            return;
        }
        let track = self.favorites_tracks[self.favorites_cursor].clone();
        let remaining: Vec<Track> = self
            .favorites_tracks
            .iter()
            .skip(self.favorites_cursor + 1)
            .cloned()
            .collect();
        self.queue = remaining;
        self.queue_cursor = 0;
        self.play_track(track).await;
    }

    pub async fn play_playlist_track(&mut self) {
        if let Some(idx) = self.viewing_playlist {
            if let Some(playlist) = self.playlists.get(idx) {
                if playlist.tracks.is_empty() {
                    return;
                }
                let track = playlist.tracks[self.playlist_track_cursor].clone();
                let remaining: Vec<Track> = playlist
                    .tracks
                    .iter()
                    .skip(self.playlist_track_cursor + 1)
                    .cloned()
                    .collect();
                self.queue = remaining;
                self.queue_cursor = 0;
                self.play_track(track).await;
            }
        }
    }

    pub fn save_queue(&self) {
        storage::save_queue(&self.queue, &self.now_playing);
    }

    pub fn settings_move_up(&mut self) {
        match self.settings_section {
            SettingsSection::Theme => {
                self.theme_cursor = self.theme_cursor.saturating_sub(1);
                self.preview_theme();
            }
            SettingsSection::Volume => {}
        }
    }

    pub fn settings_move_down(&mut self) {
        match self.settings_section {
            SettingsSection::Theme => {
                if self.theme_cursor < THEME_PRESETS.len() - 1 {
                    self.theme_cursor += 1;
                }
                self.preview_theme();
            }
            SettingsSection::Volume => {}
        }
    }

    fn preview_theme(&mut self) {
        let name = THEME_PRESETS[self.theme_cursor];
        self.theme = Theme::from_preset(name);
    }

    pub fn settings_select(&mut self) {
        match self.settings_section {
            SettingsSection::Theme => {
                let name = THEME_PRESETS[self.theme_cursor];
                self.theme = Theme::from_preset(name);
                self.current_theme_name = name.to_string();
                let _ = config::save_theme_preset(name);
                self.notify(format!("Theme set to: {}", name));
            }
            SettingsSection::Volume => {}
        }
    }

    pub fn settings_next_section(&mut self) {
        self.settings_section = match self.settings_section {
            SettingsSection::Theme => SettingsSection::Volume,
            SettingsSection::Volume => SettingsSection::Theme,
        };
        self.settings_cursor = match self.settings_section {
            SettingsSection::Theme => 0,
            SettingsSection::Volume => 0,
        };
    }

    pub fn settings_volume_up(&mut self) {
        self.player_status.volume = (self.player_status.volume + 5).min(100);
        let _ = config::save_volume(self.player_status.volume as i32);
    }

    pub fn settings_volume_down(&mut self) {
        self.player_status.volume = (self.player_status.volume - 5).max(0);
        let _ = config::save_volume(self.player_status.volume as i32);
    }

    pub fn in_settings(&self) -> bool {
        self.active_panel == Panel::Content && self.selected_library_item() == LibraryItem::Settings
    }
}
