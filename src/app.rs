use std::env;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use color_eyre::Result;
use nucleo::{Config, Nucleo};
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use arboard::Clipboard as ArboardClipboard;

mod actions;
mod fuzzy;
mod input;

pub enum Modes {
    Search,
    Action,
    Rename,
    DeleteConfirm,
}

pub struct App {
    pub input: String,
    pub character_index: usize,
    pub nucleo: Nucleo<String>,
    pub current_dir: PathBuf,
    pub mode: Modes,
    pub action_target: PathBuf,
    pending_nucleo: Option<mpsc::Receiver<Nucleo<String>>>,
    pub glob_results: Vec<String>,
    glob_receiver: Option<mpsc::Receiver<Vec<String>>>,
    clipboard: Option<ArboardClipboard>,
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
            mode: Modes::Search,
            action_target: PathBuf::new(),
            pending_nucleo: None,
            glob_results: Vec::new(),
            glob_receiver: None,
            clipboard: ArboardClipboard::new().ok(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            self.swap_nucleo();
            self.poll_glob_results();
            self.nucleo.tick(10);
            terminal.draw(|frame| crate::ui::render(&mut self, frame))?;

            if !event::poll(std::time::Duration::from_millis(16))? {
                continue;
            }

            let Some(key) = event::read()?.as_key_press_event() else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(KeyModifiers::CONTROL)
            {
                return Ok(());
            }

            match self.mode {
                Modes::Action => match key.code {
                    KeyCode::Char('o') => {
                        Self::open_file(&self.action_target);
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('y') => {
                        self.copy_path(&self.action_target.clone());
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('r') => {
                        if let Some(name) = self.action_target.file_name() {
                            self.input = name.to_string_lossy().to_string();
                            self.character_index = self.input.len();
                        }
                        self.mode = Modes::Rename;
                    }
                    KeyCode::Char('d') => self.mode = Modes::DeleteConfirm,
                    KeyCode::Esc => self.mode = Modes::Search,
                    _ => {}
                },
                Modes::Rename => match key.code {
                    KeyCode::Enter => {
                        let new_path = self.current_dir.join(&self.input);
                        std::fs::rename(&self.action_target, &new_path).ok();
                        self.reload_dir();
                        self.clear_input();
                        self.mode = Modes::Search;
                    }
                    KeyCode::Esc => {
                        self.clear_input();
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char(c) => self.insert_char(c),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_left(1),
                    KeyCode::Right => self.move_right(1),
                    _ => {}
                },
                Modes::DeleteConfirm => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        Self::delete_entry(&self.action_target);
                        self.reload_dir();
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = Modes::Search;
                    }
                    _ => {}
                },
                Modes::Search => match key.code {
                    KeyCode::Char('w')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.delete_until_whitespace();
                    }
                    KeyCode::Enter => self.select_entry(&self.input.clone()),
                    KeyCode::Char(c) => self.insert_char_and_search(c),
                    KeyCode::Backspace => self.delete_char_and_search(),
                    KeyCode::Left => self.move_left(1),
                    KeyCode::Right => self.move_right(1),
                    _ => {}
                },
            }
        }
    }
}
