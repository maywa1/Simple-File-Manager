use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;

use nucleo::{Config, Nucleo};
use nucleo::pattern::{CaseMatching, Normalization};
use walkdir::WalkDir;

use crate::app::App;

impl App {
    pub(crate) fn set_dir(nucleo: &Nucleo<String>, dir: impl AsRef<Path>) {
        let injector = nucleo.injector();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_string_lossy().to_string();
            injector.push(path, |s, cols| {
                cols[0] = s.as_str().into();
            });
        }
    }

    pub(crate) fn swap_nucleo(&mut self) {
        if let Some(ref rx) = self.pending_nucleo {
            if let Ok(new_nucleo) = rx.try_recv() {
                self.nucleo = new_nucleo;
                self.pending_nucleo = None;
                self.update_query();
            }
        }
    }

    pub(crate) fn change_dir(&mut self, dir: PathBuf) {
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

    pub(crate) fn reload_dir(&mut self) {
        self.change_dir(self.current_dir.clone());
    }

    pub(crate) fn poll_glob_results(&mut self) {
        if let Some(ref rx) = self.glob_receiver {
            if let Ok(items) = rx.try_recv() {
                self.glob_results = items;
                self.glob_receiver = None;
            }
        }
    }

    pub(crate) fn update_query(&mut self) {
        if self.input.contains('*') {
            // TODO: consider filtering using nucleo itself instead of fetching everything, but idk how yet
            self.nucleo.pattern.reparse(
                0,
                "",
                CaseMatching::Ignore,
                Normalization::Smart,
                false,
            );
            self.nucleo.tick(0);

            let pattern = self.input.clone();
            let dir = self.current_dir.clone();
            let all_items: Vec<String> = self
                .nucleo
                .snapshot()
                .matched_items(..)
                .map(|item| item.data.clone())
                .collect();
            let (tx, rx) = mpsc::channel();

            std::thread::spawn(move || {
                let glob = pattern.trim();
                let current_dir_only = !glob.contains('/');
                let items: Vec<String> = all_items
                    .iter()
                    .filter_map(|full_path| {
                        let path = Path::new(full_path);
                        let display = path
                            .strip_prefix(&dir)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        if current_dir_only && display.contains('/') {
                            return None;
                        }
                        if matches_glob(glob, &display) {
                            Some(display)
                        } else {
                            None
                        }
                    })
                    .take(30)
                    .collect();
                tx.send(items).ok();
            });

            self.glob_receiver = Some(rx);
        } else {
            self.nucleo.pattern.reparse(
                0,
                &self.input,
                CaseMatching::Ignore,
                Normalization::Smart,
                false,
            );
            self.nucleo.tick(0);
        }
    }
}

fn matches_glob(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == path;
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    if !parts[0].is_empty() && !path.starts_with(parts[0]) {
        return false;
    }
    if !parts[parts.len() - 1].is_empty() && !path.ends_with(parts[parts.len() - 1]) {
        return false;
    }

    let mut search_start = if parts[0].is_empty() { 0 } else { parts[0].len() };
    for i in 1..parts.len() - 1 {
        if let Some(pos) = path[search_start..].find(parts[i]) {
            search_start += pos + parts[i].len();
        } else {
            return false;
        }
    }

    true
}
