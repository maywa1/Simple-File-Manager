use std::path::Path;
use std::fs;

use ratatui::DefaultTerminal;

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

    pub(crate) fn open_file(&mut self, path: &Path, terminal: &mut DefaultTerminal) {
        self.mode = Modes::FileOpen;

        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .status();

        let _ = terminal.clear(); // WARN: kinda awkward writing a ui thing in here, but it's needed so that the ui does not bleed maybe I could use a helper method or something that handles everything, but that would envolve changing too much code

        self.mode = Modes::Search;
    }

    pub(crate) fn finish_delete(&mut self) {
        if self.action_target.is_dir() {
            std::fs::remove_dir_all(&self.action_target).ok();
        } else {
            std::fs::remove_file(&self.action_target).ok();
        }
        self.reload_dir();
        self.mode = Modes::Search;
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

    pub(crate) fn begin_rename(&mut self) {
        if let Some(name) = self.action_target.file_name() {
            self.input = name.to_string_lossy().to_string();
            self.character_index = self.input.len();
        }
        self.mode = Modes::Rename;
    }

    pub(crate) fn finish_rename(&mut self) {
        let new_path = self.current_dir.join(&self.input);
        std::fs::rename(&self.action_target, &new_path).ok();
        self.reload_dir();
        self.clear_input();
        self.mode = Modes::Search;
    }

    pub(crate) fn finish_move(&mut self) {
        let dest = self.current_dir.clone();
        match Self::move_file_or_dir(&self.action_target, &dest) {
            Ok(()) => {
                self.status = Some(format!("Moved to {}", dest.display()));
            }
            Err(e) => {
                self.status = Some(format!("Move failed: {e}"));
            }
        }
        self.moving = false;
        self.clear_input();
        self.reload_dir();
    }

    pub(crate) fn finish_create_dir(&mut self) {
        Self::create_dir(&self.action_target);
        self.change_dir(self.action_target.clone());
        self.reload_dir();
        self.mode = Modes::Search;
    }

    pub(crate) fn finish_create_file(&mut self) {
        Self::create_file(&self.action_target);
        self.reload_dir();
        self.mode = Modes::Search;
    }

    pub(crate) fn begin_bulk_action(&mut self) {
        self.bulk_targets = self.expand_glob();
        self.clear_input();
        self.mode = Modes::BulkAction;
    }

    pub(crate) fn bulk_delete(&mut self) {
        let count = self.bulk_targets.len();
        for path in self.bulk_targets.drain(..) {
            if path.is_dir() {
                std::fs::remove_dir_all(&path).ok();
            } else {
                std::fs::remove_file(&path).ok();
            }
        }
        self.reload_dir();
        self.status = Some(format!("Deleted {count} files"));
        self.mode = Modes::Search;
    }

    pub(crate) fn bulk_copy_paths(&mut self) {
        let paths: Vec<String> = self
            .bulk_targets
            .iter()
            .filter_map(|p| p.to_str().map(String::from))
            .collect();
        let text = paths.join("\n");
        if let Some(clipboard) = self.clipboard.as_mut() {
            let _ = clipboard.set_text(&text);
        }
        self.status = Some(format!("Copied {} paths to clipboard", paths.len()));
        self.bulk_targets.clear();
        self.mode = Modes::Search;
    }

    pub(crate) fn begin_bulk_move(&mut self) {
        self.moving = true;
        self.clear_input();
        self.mode = Modes::Search;
    }

    pub(crate) fn finish_bulk_move(&mut self) {
        let dest = self.current_dir.clone();
        let mut moved = 0;
        let mut errors = 0;
        for source in self.bulk_targets.drain(..) {
            match Self::move_file_or_dir(&source, &dest) {
                Ok(()) => moved += 1,
                Err(_) => errors += 1,
            }
        }
        if errors == 0 {
            self.status = Some(format!("Moved {moved} files to {}", dest.display()));
        } else {
            self.status = Some(format!("Moved {moved}, {errors} failed"));
        }
        self.moving = false;
        self.clear_input();
        self.reload_dir();
    }

    pub(crate) fn move_file_or_dir(source: &Path, destination: &Path) -> Result<(), String> {
        let dest_path = if destination.is_dir() {
            destination.join(
                source.file_name().ok_or_else(|| "Could not get file name".to_string())?,
            )
        } else {
            destination.to_path_buf()
        };

        if let Err(e) = fs::rename(source, &dest_path) {
            if e.kind() == std::io::ErrorKind::Other {
                fs::copy(source, &dest_path).map_err(|e| e.to_string())?;
                fs::remove_file(source).map_err(|e| e.to_string())?;
            } else {
                return Err(e.to_string());
            }
        }
        Ok(())
    }
}
