use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use nucleo::Nucleo;
use nucleo::Config;
use nucleo::pattern::{CaseMatching, Normalization};
use walkdir::WalkDir;

pub struct App {
    pub input: String,
    pub character_index: usize,
    pub nucleo: Nucleo<String>,
    pub current_dir: PathBuf,
    pending_nucleo: Option<mpsc::Receiver<Nucleo<String>>>,
}

impl App {
    pub fn new() -> Self {
        let nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let current_dir = env::current_dir().expect("Could not read directory");

        Self::set_dir(&nucleo, &current_dir);

        Self {
            input: String::new(),
            character_index: 0,
            nucleo,
            current_dir,
            pending_nucleo: None,
        }
    }

    pub fn set_dir(nucleo: &Nucleo<String>, dir: impl AsRef<std::path::Path>) {
        let injector = nucleo.injector();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_string_lossy().to_string();
            injector.push(path, |s, cols| {
                cols[0] = s.as_str().into();
            });
        }
    }

    fn swap_nucleo(&mut self) {
        if let Some(ref rx) = self.pending_nucleo {
            if let Ok(new_nucleo) = rx.try_recv() {
                self.nucleo = new_nucleo;
                self.pending_nucleo = None;
                self.update_query();
            }
        }
    }

    fn change_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;

        let new_nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let injector = new_nucleo.injector();
        let scan_dir = self.current_dir.clone();
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            for entry in WalkDir::new(scan_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path().to_string_lossy().to_string();
                injector.push(path, |s, cols| {
                    cols[0] = s.as_str().into();
                });
            }
            tx.send(new_nucleo).ok();
        });

        self.pending_nucleo = Some(rx);
    }

    fn navigate_to(&mut self, path: &str) {
        let target = self.current_dir.join(path);
        if target.is_dir() {
            self.change_dir(target);
            self.clear_input();
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            self.swap_nucleo();
            self.nucleo.tick(10);
            terminal.draw(|frame| crate::ui::render(&mut self, frame))?;

            if event::poll(Duration::from_millis(16))? {
                if let Some(key) = event::read()?.as_key_press_event() {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.delete_until_whitespace();
                        }
                        KeyCode::Enter => self.navigate_to(&self.input.clone()),
                        KeyCode::Char(c) => self.insert_char_and_search(c),
                        KeyCode::Backspace => self.delete_char_and_search(),
                        KeyCode::Left => self.move_left(1),
                        KeyCode::Right => self.move_right(1),
                        _ => {}
                    }
                }
            }
        }
    }

    fn delete_until_whitespace(&mut self){
        let mut cursor = self.character_index;

        let bytes = self.input.as_bytes();

        while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
            cursor -= 1;
        }

        let start = cursor;
        while cursor > 0 && !bytes[cursor - 1].is_ascii_whitespace() {
            cursor -= 1;
        }

        let word_start = cursor;
        let word_end = start;

        self.input.replace_range(word_start..word_end, "");

        self.character_index = word_start;
        self.update_query();
    }
    pub fn go_to_parent_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.change_dir(parent.to_path_buf());
        }
    }

    fn insert_char_and_search(&mut self, c: char) {
        let idx = self.byte_index();
        self.input.insert(idx, c);
        self.move_right(1);

        if self.input.trim() == ".." {
            self.go_to_parent_dir();
            self.clear_input();
            return;
        }

        if c == '/' {
            let path = self.input.trim().trim_end_matches('/').to_string();
            if !path.is_empty() {
                self.navigate_to(&path);
                return;
            }
        }

        self.update_query();
    }

    fn delete_char_and_search(&mut self) {
        if self.character_index == 0 {
            return;
        }

        let idx = self.character_index - 1;
        let byte_idx = self
            .input
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.remove(byte_idx);
        self.move_left(1);
        self.update_query();
    }

    fn translate_glob(input: &str) -> String {
        if !input.contains('*') {
            return input.to_string();
        }

        let trimmed = input.trim();

        if trimmed == "*" {
            return String::new();
        }

        if trimmed.starts_with('*') && trimmed.ends_with('*') && trimmed.len() > 2 {
            let inner = &trimmed[1..trimmed.len() - 1];
            return format!("'{inner}");
        }

        if let Some(suffix) = trimmed.strip_prefix('*') {
            return format!("{suffix}$");
        }

        if let Some(prefix) = trimmed.strip_suffix('*') {
            return format!("^{prefix}");
        }

        input.to_string()
    }

    fn update_query(&mut self) {
        let pattern = Self::translate_glob(&self.input);
        self.nucleo.pattern.reparse(
            0,
            &pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            false,
        );
        self.nucleo.tick(0);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn move_left(&mut self, quantity: usize) {
        self.character_index = self.character_index.saturating_sub(quantity);
    }

    fn move_right(&mut self, quantity: usize) {
        self.character_index = (self.character_index + quantity).min(self.input.len());
    }

    fn clear_input(&mut self){
        self.input = String::new();
        self.character_index = 0;
    }
}
