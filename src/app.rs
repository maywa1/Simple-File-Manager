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
    CreateFileOrDir,
    FileOpen,
    BulkAction,
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
    pub moving: bool,
    pub status: Option<String>,
    pub bulk_targets: Vec<PathBuf>,
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
            moving: false,
            status: None,
            bulk_targets: Vec::new(),
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

            self.status = None;

            match self.mode {
                Modes::Action => match key.code {
                    KeyCode::Char('o') => {
                        self.open_file(&self.action_target.clone(), terminal);
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('y') => {
                        self.copy_path(&self.action_target.clone());
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('r') => self.begin_rename(),
                    KeyCode::Char('m') => {
                        self.moving = true;
                        self.clear_input();
                        self.mode = Modes::Search;
                    }
                    KeyCode::Char('d') => self.mode = Modes::DeleteConfirm,
                    KeyCode::Esc => self.mode = Modes::Search,
                    _ => {}
                },
                Modes::Rename => match key.code {
                    KeyCode::Enter => self.finish_rename(),
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
                    KeyCode::Char('y') | KeyCode::Char('Y') => self.finish_delete(),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = Modes::Search;
                    }
                    _ => {}
                },
                Modes::CreateFileOrDir => match key.code {
                    KeyCode::Char('d') | KeyCode::Char('D') => self.finish_create_dir(),
                    KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Esc => self.finish_create_file(),
                    _ => {}
                },
                Modes::Search => match key.code {
                    KeyCode::Char('w')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.delete_until_whitespace();
                    }
                    KeyCode::Enter => {
                        if self.moving {
                            if !self.bulk_targets.is_empty() {
                                self.finish_bulk_move();
                            } else {
                                self.finish_move();
                            }
                        } else if self.input.contains('*') && !self.glob_results.is_empty() {
                            self.begin_bulk_action();
                        } else {
                            self.select_entry(&self.input.clone());
                        }
                    }
                    KeyCode::Esc => {
                        if self.moving {
                            self.moving = false;
                            self.clear_input();
                        }
                    }
                    KeyCode::Tab => self.auto_complete_and_search(),
                    KeyCode::Char(c) => self.insert_char_and_search(c),
                    KeyCode::Backspace => self.delete_char_and_search(),
                    KeyCode::Left => self.move_left(1),
                    KeyCode::Right => self.move_right(1),
                    _ => {}
                },
                Modes::BulkAction => match key.code {
                    KeyCode::Char('d') | KeyCode::Char('D') => self.bulk_delete(),
                    KeyCode::Char('y') => self.bulk_copy_paths(),
                    KeyCode::Char('m') => self.begin_bulk_move(),
                    KeyCode::Esc => {
                        self.bulk_targets.clear();
                        self.mode = Modes::Search;
                    }
                    _ => {}
                },
                Modes::FileOpen => {},
            }
        }
    }
}

