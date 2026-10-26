use serde::{Deserialize, Serialize};
use std::{fs, io::{self, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use crossterm::{cursor::MoveTo, event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    front: String,
    back: String,
    due_at: u64,
    interval_days: u32,
    repetitions: u32,
    ease: f32
}

impl Card {
    fn new(front: String, back: String) -> Self {
        Self {
            front,
            back,
            due_at: now(),
            interval_days: 0,
            repetitions: 0,
            ease: 2.5
        }
    } 
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
} 

fn data_path() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));

    home.join(".local/share/imret/cards.json")
}

fn load_cards(path: &PathBuf) -> io::Result<Vec<Card>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;
    let cards = serde_json::from_str(&contents).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok(cards)
}

fn save_cards(path: &PathBuf, cards: &[Card]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(cards).map_err(|error| io::Error::other(error))?;

    fs::write(path, contents)
}

fn main() -> io::Result<()> {
    let path = data_path();
    let cards = load_cards(&path)?;
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let result = run_menu(&mut stdout, cards.len());

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    result
}

fn run_menu(stdout: &mut io::Stdout, card_count: usize) -> io::Result<()> {
    loop {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        writeln!(stdout, "imret flashcards")?;
        writeln!(stdout)?;
        writeln!(stdout, "number of cards: {}", card_count)?;
        writeln!(stdout)?;

        writeln!(stdout, "[1] create cards")?;
        writeln!(stdout, "[2] review cards")?;
        writeln!(stdout, "[q] quit")?;
        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    Ok(())
}
