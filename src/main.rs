mod app;
mod codec;
mod db;
mod ui;
mod update;

use app::{App, DetailTab, InputMode, View};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::{Path, PathBuf};
use age::secrecy::Secret;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// TUI explorer for YouTube transcript summaries stored in SQLite
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the SQLite database file (deprecated/fallback if no subcommand)
    #[arg(short, long)]
    db: Option<PathBuf>,
    
    /// Password for encrypted database files
    #[arg(short, long)]
    password: Option<String>,
}

const DEFAULT_DB_URL: &str = "https://rocketrecap.com/exports/summaries20260123.age";

#[derive(Subcommand)]
enum Commands {
    /// Run the TUI explorer (default)
    Run {
        /// Path to the SQLite database file
        #[arg(short, long)]
        db: PathBuf,
    },
    /// Compress and encrypt a database file
    Encrypt {
        /// Input database file
        #[arg(short, long)]
        input: PathBuf,
        /// Output encrypted file
        #[arg(short, long)]
        output: PathBuf,
        /// Fast compression (quality 1) works best for speed
        #[arg(long, conflicts_with = "best")]
        fast: bool,
        /// Best compression (quality 11) works best for size but is slow
        #[arg(long, conflicts_with = "fast")]
        best: bool,
    },
    /// Decrypt and decompress a database file
    Decrypt {
        /// Input encrypted file
        #[arg(short, long)]
        input: PathBuf,
        /// Output database file
        #[arg(short, long)]
        output: PathBuf,
        /// Password for decryption
        #[arg(short, long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Normalize command
    let command = match cli.command {
        Some(c) => c,
        None => {
            if let Some(db_path) = cli.db {
                Commands::Run { db: db_path }
            } else {
                // Default behavior: download if needed and run
                let project_dirs = directories::ProjectDirs::from("com", "rocketrecap", "transcript-explorer")
                    .ok_or("Could not determine home directory")?;
                let db_path = project_dirs.cache_dir().join("summaries20260123.age");
                Commands::Run { db: db_path }
            }
        }
    };

    match command {
        Commands::Encrypt { input, output, fast, best } => {
            if !input.exists() {
                eprintln!("Error: input file not found: {}", input.display());
                std::process::exit(1);
            }
            
            let quality = if fast {
                1
            } else if best {
                11
            } else {
                6 // Default
            };

            let password = if let Some(p) = cli.password.as_ref() {
                p.clone()
            } else {
                eprint!("Enter password: ");
                read_password_with_stars()?
            };
            eprintln!("Encrypting {} -> {} (quality: {})...", input.display(), output.display(), quality);
            codec::encrypt_stream(&input, &output, Secret::new(password), quality)?;
            eprintln!("Done.");
        }
        Commands::Decrypt { input, output, password } => {
            if !input.exists() {
                eprintln!("Error: input file not found: {}", input.display());
                std::process::exit(1);
            }
            let password = if let Some(p) = password {
                p
            } else if let Some(p) = cli.password.as_ref() {
                p.clone()
            } else {
                eprint!("Enter password: ");
                read_password_with_stars()?
            };
            eprintln!("Decrypting {} -> {} ...", input.display(), output.display());
            codec::decrypt_stream(&input, &output, Secret::new(password))?;
            eprintln!("Done.");
        }
        Commands::Run { db } => {
            let mut db_path = db;
            
            // Check if DB exists, if not try to find it in cache or download it
            if !db_path.exists() {
                let project_dirs = directories::ProjectDirs::from("com", "rocketrecap", "transcript-explorer")
                    .ok_or("Could not determine home directory")?;
                let cache_dir = project_dirs.cache_dir();
                std::fs::create_dir_all(cache_dir)?;
                
                // If it's the default name or doesn't exist, we might want to check the cache
                let cached_path = cache_dir.join("summaries20260123.age");
                
                if cached_path.exists() {
                    db_path = cached_path;
                } else if db_path.file_name().map_or(false, |n| n == "summaries20260123.age") || !db_path.exists() {
                    eprintln!("Database not found. Downloading from {}...", DEFAULT_DB_URL);
                    download_db(DEFAULT_DB_URL, &cached_path).await?;
                    db_path = cached_path;
                } else {
                    eprintln!("Error: database file not found: {}", db_path.display());
                    std::process::exit(1);
                }
            }

            // Check if it's potentially encrypted (basic check or user invoked)
            // We assume if it fails to open as SQLite or has extension .age, we try to decrypt?
            // Or we just try to read the header.
            // For now, let's look for known extensions or just try to open it as SQLite first?
            // Actually, we can just check the header bytes. SQLite header is "SQLite format 3\0".
            // Age header is "age-encryption.org".
            
            let mut is_encrypted = false;
            if let Ok(mut file) = std::fs::File::open(&db_path) {
                use std::io::Read;
                let mut buffer = [0u8; 18]; // "age-encryption.org" length
                if file.read_exact(&mut buffer).is_ok() {
                     if &buffer == b"age-encryption.org" {
                         is_encrypted = true;
                     }
                }
            }
            
            let _temp_file; // Keep alive until function end
            
            let target_db_path = if is_encrypted {
                eprintln!("Detected encrypted database: {}", db_path.display());
                let password = if let Some(p) = cli.password {
                    p
                } else {
                    eprint!("Enter password: ");
                    read_password_with_stars()?
                };
                
                eprintln!("Decrypting to temporary file...");
                let temp = tempfile::NamedTempFile::new()?;
                codec::decrypt_stream(&db_path, temp.path(), Secret::new(password))?;
                
                _temp_file = temp; // extend lifetime
                _temp_file.path().to_path_buf()
            } else {
                db_path
            };

            // Open database
            let database = db::Database::open(&target_db_path).await?;

            // Create app
            let mut app = App::new(database);
            app.init().await?;

            // Spawn background update thread
            let _update_thread = {
                match update::UpdateConfiguration::load() {
                    Ok(config) => {
                        match update::UpdateManager::new(config) {
                            Ok(manager) => {
                                Some(manager.spawn_background_thread())
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to initialize update manager: {}", e.user_message());
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load update configuration: {}", e.user_message());
                        None
                    }
                }
            };

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
        }
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
        View::Filters => handle_filters_key(app, key).await?,
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
        KeyCode::Char('f') => {
            app.view = View::Filters;
            app.status_msg.clear();
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
        KeyCode::Char('y') => {
            if let Some(group) = app.grouped_similar_results.get(app.similar_selected) {
                if let Some(res) = group.items.first() {
                    let link = &res.original_source_link;
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
        }
        KeyCode::Char('o') => {
            if let Some(group) = app.grouped_similar_results.get(app.similar_selected) {
                if let Some(res) = group.items.first() {
                    let link = &res.original_source_link;
                    if !link.is_empty() {
                        let _ = std::process::Command::new("xdg-open").arg(link).spawn();
                        app.status_msg = format!("Opening: {}", link);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn download_db(url: &str, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let total_size = response.content_length().ok_or("Failed to get content length")?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let mut file = std::fs::File::create(output)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Download complete");
    Ok(())
}

fn read_password_with_stars() -> Result<String, Box<dyn std::error::Error>> {
    let mut password = String::new();
    enable_raw_mode()?;

    let res = (|| -> Result<String, Box<dyn std::error::Error>> {
        loop {
            if let Event::Key(event) = event::read()? {
                if event.kind != KeyEventKind::Release {
                    match event.code {
                        KeyCode::Enter => {
                            eprintln!();
                            break;
                        }
                        KeyCode::Backspace => {
                            if !password.is_empty() {
                                password.pop();
                                // Move back, print space, move back again to clear character
                                eprint!("\u{0008} \u{0008}");
                                std::io::stderr().flush()?;
                            }
                        }
                        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                            eprintln!();
                            return Err("Interrupted by user".into());
                        }
                        KeyCode::Char(c) => {
                            password.push(c);
                            eprint!("*");
                            std::io::stderr().flush()?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(password)
    })();

    disable_raw_mode()?;
    res
}

async fn handle_filters_key(app: &mut crate::app::App, key: event::KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
    use crate::app::FilterBuilderState;
    use crossterm::event::KeyCode;

    match app.filter_builder_state.clone() {
        FilterBuilderState::Inactive => {
             match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.view = crate::app::View::List;
                }
                KeyCode::Char('a') => {
                    app.filter_builder_state = FilterBuilderState::SelectingField;
                    app.status_msg = "Select field: (c)ost, (i)nput, (o)utput, (m)odel, (h)ost".to_string();
                }
                KeyCode::Char('d') => {
                    app.clear_global_filters();
                    app.status_msg = "Global filters cleared".to_string();
                }
                _ => {}
            }
        }
        FilterBuilderState::SelectingField => {
            match key.code {
                KeyCode::Char('c') => {
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field: "cost".to_string(), step: 0, buffer: String::new(), min_val: 0.0 };
                    app.status_msg = "Enter MIN cost (default 0):".to_string();
                }
                KeyCode::Char('i') => {
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field: "input_tokens".to_string(), step: 0, buffer: String::new(), min_val: 0.0 };
                    app.status_msg = "Enter MIN input tokens (default 0):".to_string();
                }
                KeyCode::Char('o') => {
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field: "output_tokens".to_string(), step: 0, buffer: String::new(), min_val: 0.0 };
                    app.status_msg = "Enter MIN output tokens (default 0):".to_string();
                }
                KeyCode::Char('m') => {
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field: "model".to_string(), step: 0, buffer: String::new(), min_val: 0.0 };
                    app.status_msg = "Enter model pattern (supports *):".to_string();
                }
                KeyCode::Char('h') => {
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field: "host".to_string(), step: 0, buffer: String::new(), min_val: 0.0 };
                    app.status_msg = "Enter host pattern (supports *):".to_string();
                }
                KeyCode::Esc => {
                    app.filter_builder_state = FilterBuilderState::Inactive;
                    app.status_msg = String::new();
                }
                _ => {}
            }
        }
        FilterBuilderState::EnteringValue { field, step, mut buffer, min_val } => {
            match key.code {
                KeyCode::Enter => {
                    match field.as_str() {
                        "cost" | "input_tokens" | "output_tokens" => {
                            if step == 0 {
                                let m = buffer.parse::<f64>().unwrap_or(0.0);
                                app.filter_builder_state = FilterBuilderState::EnteringValue { field: field.clone(), step: 1, buffer: String::new(), min_val: m };
                                app.status_msg = format!("Min: {}. Enter MAX (default max):", m);
                            } else {
                                let max = if buffer.is_empty() {
                                    app.field_stats.get(&field).map(|s| s.max).unwrap_or(f64::MAX)
                                } else {
                                    buffer.parse::<f64>().unwrap_or(f64::MAX)
                                };
                                app.add_filter(crate::app::Filter::Range { field: field.clone(), min: min_val, max });
                                app.filter_builder_state = FilterBuilderState::Inactive;
                                app.status_msg = format!("Added filter: {} in range [{}, {}]", field, min_val, max);
                            }
                        }
                        "model" | "host" => {
                            let pattern = if buffer.is_empty() { "*".to_string() } else { buffer };
                            app.add_filter(crate::app::Filter::Match { field: field.clone(), pattern: pattern.clone() });
                            app.filter_builder_state = FilterBuilderState::Inactive;
                            app.status_msg = format!("Added filter: {} matches '{}'", field, pattern);
                        }
                        _ => {
                             app.filter_builder_state = FilterBuilderState::Inactive;
                        }
                    }
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field, step, buffer, min_val };
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                    app.filter_builder_state = FilterBuilderState::EnteringValue { field, step, buffer, min_val };
                }
                KeyCode::Esc => {
                    app.filter_builder_state = FilterBuilderState::Inactive;
                    app.status_msg = String::new();
                }
                _ => {}
            }
        }
    }
    Ok(())
}
