mod app;
mod db;
mod ui;

use app::{App, DetailTab, InputMode, View};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

/// TUI explorer for YouTube transcript summaries stored in SQLite
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the SQLite database file
    #[arg(short, long)]
    db: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Verify DB file exists
    if !cli.db.exists() {
        eprintln!("Error: database file not found: {}", cli.db.display());
        std::process::exit(1);
    }

    // Open database
    let database = db::Database::open(&cli.db).await?;

    // Create app
    let mut app = App::new(database);
    app.init().await?;

    // Init terminal
    let mut terminal = ratatui::init();

    // Initial page size setup
    let size = terminal.size()?;
    app.update_page_size(size.height);

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    ratatui::restore();

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| ui::render(app, frame))?;

        if app.should_quit {
            return Ok(());
        }

        // Poll for events with a 250ms timeout
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    handle_key(app, key).await?;
                }
                Event::Resize(_, height) => {
                    app.update_page_size(height);
                }
                _ => {}
            }
        }
    }
}

async fn handle_key(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Help toggle (global)
    if key.code == KeyCode::Char('?') && app.input_mode == InputMode::Normal {
        app.show_help = !app.show_help;
        return Ok(());
    }

    // If help is showing, any key closes it
    if app.show_help {
        app.show_help = false;
        return Ok(());
    }

    // Ctrl+C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return Ok(());
    }

    // Handle based on input mode and view
    if app.input_mode == InputMode::Editing {
        handle_filter_input(app, key).await?;
        return Ok(());
    }

    match app.view {
        View::List => handle_list_key(app, key).await?,
        View::Detail => handle_detail_key(app, key).await?,
        View::Similar => handle_similar_key(app, key).await?,
    }

    Ok(())
}

async fn handle_filter_input(
    app: &mut App,
    key: KeyEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut changed = false;
    match key.code {
        KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.filter.pop();
            changed = true;
        }
        KeyCode::Char(c) => {
            app.filter.push(c);
            changed = true;
        }
        _ => {}
    }
    
    if changed {
        app.apply_filter();
    }
    Ok(())
}

async fn handle_list_key(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Editing;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.list_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.list_prev();
        }
        KeyCode::PageDown => {
            app.list_page_down();
        }
        KeyCode::PageUp => {
            app.list_page_up();
        }
        KeyCode::Char(' ') => {
            app.toggle_expand();
        }
        KeyCode::Enter => {
            app.open_detail().await?;
        }
        KeyCode::Char('s') => {
            app.open_similar().await?;
        }
        KeyCode::Char('g') => {
            // Jump to first page
            app.list_offset = 0;
            app.list_selected = 0;
            app.update_list_page();
        }
        KeyCode::Char('G') => {
            // Jump to last page
            if !app.grouped_items.is_empty() {
                let last_page_start = (app.grouped_items.len().saturating_sub(1) / app.page_size) * app.page_size;
                app.list_offset = last_page_start;
                app.update_list_page();
                app.list_selected = app.list_items.len().saturating_sub(1);
            }
        }
        KeyCode::Esc => {
            // Clear filter
            if !app.filter.is_empty() {
                app.filter.clear();
                app.apply_filter();
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_detail_key(
    app: &mut App,
    key: KeyEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.view = View::List;
            app.detail = None;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_down();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_up();
        }
        KeyCode::PageDown => {
            app.scroll_page_down();
        }
        KeyCode::PageUp => {
            app.scroll_page_up();
        }
        KeyCode::Tab => {
            app.detail_tab = app.detail_tab.next();
            app.detail_scroll = 0;
        }
        KeyCode::BackTab => {
            app.detail_tab = app.detail_tab.prev();
            app.detail_scroll = 0;
        }
        KeyCode::Char('1') => {
            app.detail_tab = DetailTab::Summary;
            app.detail_scroll = 0;
        }
        KeyCode::Char('2') => {
            app.detail_tab = DetailTab::Transcript;
            app.detail_scroll = 0;
        }
        KeyCode::Char('3') => {
            app.detail_tab = DetailTab::Timestamps;
            app.detail_scroll = 0;
        }
        KeyCode::Char('s') => {
            app.open_similar().await?;
        }
        KeyCode::Char('y') => {
            if let Some(ref detail) = app.detail {
                // Try to copy link to clipboard using xclip/xsel/wl-copy
                let link = &detail.original_source_link;
                if !link.is_empty() {
                    if let Ok(mut child) = std::process::Command::new("xclip")
                        .args(["-selection", "clipboard"])
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        use std::io::Write;
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(link.as_bytes());
                        }
                        let _ = child.wait();
                        app.status_msg = format!("Copied: {}", link);
                    } else if let Ok(mut child) = std::process::Command::new("wl-copy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        use std::io::Write;
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(link.as_bytes());
                        }
                        let _ = child.wait();
                        app.status_msg = format!("Copied: {}", link);
                    } else {
                        app.status_msg = format!("Link: {} (clipboard not available)", link);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_similar_key(
    app: &mut App,
    key: KeyEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.view = View::List;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.grouped_similar_results.is_empty()
                && app.similar_selected + 1 < app.grouped_similar_results.len()
            {
                app.similar_selected += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.similar_selected > 0 {
                app.similar_selected -= 1;
            }
        }
        KeyCode::Char(' ') => {
            app.toggle_expand();
        }
        KeyCode::Enter => {
            app.open_detail().await?;
        }
        _ => {}
    }
    Ok(())
}
