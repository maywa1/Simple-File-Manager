use std::path::Path;
use std::fs;

use crate::app::{App, Modes};

impl App {
    pub(crate) fn select_entry(&mut self, path: &str) {
        let target = self.current_dir.join(path);
        if target.exists() {
            self.action_target = target;
            self.clear_input();
            self.mode = crate::app::Modes::Action;
        } else {
            self.action_target = target;
            self.clear_input();
            self.mode = crate::app::Modes::CreateFileOrDir;
        }
    }

    pub(crate) fn open_file(&mut self, path: &Path) {
        self.mode = Modes::FileOpen;

        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .status();

        self.mode = Modes::Search;
    }

    pub(crate) fn delete_entry(path: &Path) {
        if path.is_dir() {
            std::fs::remove_dir_all(path).ok();
        } else {
            std::fs::remove_file(path).ok();
        }
    }

    pub(crate) fn copy_path(&mut self, path: &Path) {
        if let Some(path_str) = path.to_str() {
            if let Some(clipboard) = self.clipboard.as_mut() {
                let _ = clipboard.set_text(path_str);
            }
        }
    }

    pub(crate) fn go_to_parent_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.change_dir(parent.to_path_buf());
        }
    }


    pub(crate) fn create_file(path: &Path) {
        fs::File::create(path).expect("Error creating file");
    }

    pub(crate) fn create_dir(path: &Path) {
        fs::create_dir(path).expect("Error creating directory");
    }
}
