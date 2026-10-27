use serde::{Deserialize, Serialize};
use std::{fs, io::{self, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use crossterm::{cursor::MoveTo, event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    front: String,
    back: String,
    #[serde(default = "default_deck")]
    deck: String,
    due_at: u64,
    interval_days: u32,
    repetitions: u32,
    ease: f32
}

impl Card {
    fn new(front: String, back: String, deck: String) -> Self {
        Self {
            front,
            back,
            deck,
            due_at: now(),
            interval_days: 0,
            repetitions: 0,
            ease: 2.5
        }
    } 
}

fn default_deck() -> String {
    String::from("Default")
}

fn deck_matches(card_deck: &str, selected_deck: &str) -> bool {
    card_deck == selected_deck || card_deck.strip_prefix(selected_deck).is_some_and(|remainder| remainder.starts_with("::"))
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
    let mut cards = load_cards(&path)?;
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let result = run_menu(&mut stdout, &mut cards, &path);

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    result
}

fn run_menu(stdout: &mut io::Stdout, cards: &mut Vec<Card>, path: &PathBuf) -> io::Result<()> {
    loop {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        let card_count = cards.iter().filter(|card| deck_matches(&card.deck, "Default")).count();

        write!(stdout, "imret flashcards\r\n\r\nnumber of cards: {}\r\n\r\n[1] create cards\r\n[2] review cards\r\n[q] quit \r\n", card_count)?;
        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('1') => create_card(cards, path)?,
                KeyCode::Char('2') => {
                    let deck = choose_deck()?;
                    review_cards(cards, path, &deck);
                }
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    Ok(())
}

fn create_card(cards: &mut Vec<Card>, path: &PathBuf) -> io::Result<()> {
    let front = prompt("front:")?;
    let back = prompt("back:")?;
    let deck = choose_deck()?;
    let reverse = prompt("reverse card? (y/n):")?;

    if front.is_empty() || back.is_empty() {
        return Ok(());
    }

    cards.push(Card::new(front.clone(), back.clone(), deck.clone()));
    if reverse.eq_ignore_ascii_case("y") {
        cards.push(Card::new(back, front, deck));
    }

    save_cards(path, cards)?;

    Ok(())
}

fn review_cards(cards: &mut Vec<Card>, path: &PathBuf, deck: &str) -> io::Result<()> {
    loop {
        let Some(index) = cards.iter().position(|card| {deck_matches(&card.deck, deck) && card.due_at <= now()}) else {
            prompt(&format!("no cards due in {}. press enter to return.", deck))?;
            return Ok(())
        };

        let front = cards[index].front.clone();
        prompt(&format!("{}", front))?;

        let back = cards[index].back.clone();
        let rating = loop {
            let answer = prompt(&format!("{back}\n\n(1) again (2) hard (3) good (4)"))?;

            match answer.parse::<u8>() {
                Ok(rating @ 1..=4) => break rating,
                _ => continue
            }
        };

        schedule(&mut cards[index], rating);
        save_cards(path, cards)?;
    }
}

fn schedule(card: &mut Card, rating: u8) {
    if rating == 1 {
        card.repetitions = 0;
        card.interval_days = 0;
        card.due_at = now() + 60;
        return;
    }
    
    let days = match (rating, card.repetitions) {
        (2, _) => (card.interval_days.max(1) as f32 * 1.2).ceil() as u32,
        (_, 0) => 1,
        (_, 1) => 3,
        _ => (card.interval_days.max(1) as f32 * card.ease).ceil() as u32
    };

    card.interval_days = days.max(1);
    card.repetitions += 1;

    if rating == 2 {
        card.ease = (card.ease - 0.15).max(1.3);
    } else if rating == 4 {
        card.ease += 0.15;
    }

    card.due_at = now() + u64::from(card.interval_days) * 24 * 60 * 60;
}

fn prompt(label: &str) -> io::Result<String> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    print!("{label} ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    
    Ok(input.trim().to_string())
}

fn choose_deck() -> io::Result<String> {
    let deck = prompt("deck:")?;

    if deck.is_empty() {
        Ok("Default".to_string)
    } else {
        Ok(deck)
    }
}
