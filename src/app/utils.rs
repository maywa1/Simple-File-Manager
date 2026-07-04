use ratatui::DefaultTerminal;

use crate::app::{App, Modes};

impl App {
    pub fn change_mode(&mut self, mode: Modes, terminal: &mut DefaultTerminal) {
        terminal.clear();
        self.mode = mode;
    }
}
