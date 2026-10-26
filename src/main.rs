use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH}
};
use std::{fs, io, path::PathBuf};

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

fn load_cards() {
    todo!()
}

fn save_cards() {
    todo!()
}
