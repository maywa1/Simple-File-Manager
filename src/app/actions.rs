use std::path::Path;

use crate::app::App;

impl App {
    pub(crate) fn select_entry(&mut self, path: &str) {
        let target = self.current_dir.join(path);
        if target.exists() {
            self.action_target = target;
            self.clear_input();
            self.mode = crate::app::Modes::Action;
        }
    }

    pub(crate) fn open_file(path: &Path) {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .ok();
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
}
