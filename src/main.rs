mod api;
mod app;
mod input;
mod player;
mod ui;

use anyhow::Result;
use crossterm::event::{Event, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("ytmusic {}", VERSION);
        return Ok(());
    }

    check_dependencies();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("Error: {:#}", e);
    }

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = app::App::new().await?;
    let mut events = EventStream::new();
    let tick_rate = std::time::Duration::from_millis(200);

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        let timeout = tokio::time::sleep(tick_rate);
        tokio::pin!(timeout);

        tokio::select! {
            event = events.next() => {
                match event {
                    Some(Ok(Event::Key(key))) => {
                        if input::handle_key(&mut app, key).await {
                            break;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    _ => {}
                }
            }
            _ = &mut timeout => {}
        }

        app.tick().await;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn check_dependencies() {
    let mpv_ok = std::process::Command::new("mpv")
        .arg("--version")
        .output()
        .is_ok();

    let ytdlp_ok = std::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
        .is_ok();

    if !mpv_ok || !ytdlp_ok {
        eprintln!("ytmusic requires the following dependencies:");
        if !mpv_ok {
            eprintln!("  - mpv (media player) — https://mpv.io");
        }
        if !ytdlp_ok {
            eprintln!("  - yt-dlp (stream extractor) — https://github.com/yt-dlp/yt-dlp");
        }
        eprintln!();
        eprintln!("Install with:");
        eprintln!("  macOS:  brew install mpv yt-dlp");
        eprintln!("  Linux:  sudo apt install mpv && pip install yt-dlp");
        eprintln!();
        std::process::exit(1);
    }
}
