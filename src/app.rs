use crate::api::{Track, YtMusicClient};
use crate::player::{MpvProcess, PlaybackState, PlayerCommand, PlayerEvent, PlayerSender, PlayerStatus};
use anyhow::Result;
use tokio::sync::mpsc;

pub enum AppEvent {
    Player(PlayerEvent),
    SearchResults(Result<Vec<Track>, String>),
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
    Queue,
}

impl LibraryItem {
    pub const ALL: [LibraryItem; 3] = [LibraryItem::Home, LibraryItem::Search, LibraryItem::Queue];

    pub fn label(&self) -> &str {
        match self {
            LibraryItem::Home => "Home",
            LibraryItem::Search => "Search",
            LibraryItem::Queue => "Queue",
        }
    }
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

    pub api: YtMusicClient,
    player_sender: Option<PlayerSender>,
    _mpv: Option<MpvProcess>,
    event_rx: mpsc::Receiver<AppEvent>,
    event_tx: mpsc::Sender<AppEvent>,
    pending_load: Option<String>,
}

impl App {
    pub async fn new() -> Result<Self> {
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

        Ok(Self {
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

            queue: Vec::new(),
            queue_cursor: 0,
            history: Vec::new(),

            now_playing: None,
            player_status: PlayerStatus::default(),
            shuffle: false,
            repeat: RepeatMode::Off,

            notification: None,

            api: YtMusicClient::new(),
            player_sender: sender,
            _mpv: mpv,
            event_rx,
            event_tx,
            pending_load: None,
        })
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
            let result = api
                .search(&query)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(AppEvent::SearchResults(result)).await;
        });
    }

    pub async fn play_track(&mut self, track: Track) {
        let url = track.youtube_url();
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
            Panel::Content => {
                self.search_result_cursor = self.search_result_cursor.saturating_sub(1)
            }
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
            Panel::Content => {
                if !self.search_results.is_empty()
                    && self.search_result_cursor < self.search_results.len() - 1
                {
                    self.search_result_cursor += 1;
                }
            }
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
            Panel::Content => self.search_result_cursor = 0,
            Panel::Queue => self.queue_cursor = 0,
        }
    }

    pub fn move_cursor_bottom(&mut self) {
        match self.active_panel {
            Panel::Library => self.library_cursor = LibraryItem::ALL.len().saturating_sub(1),
            Panel::Content => {
                self.search_result_cursor = self.search_results.len().saturating_sub(1)
            }
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
}
