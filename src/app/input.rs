use crate::app::App;

impl App {
    pub(crate) fn insert_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.input.insert(idx, c);
        self.move_right(1);
    }

    pub(crate) fn delete_char(&mut self) {
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
    }

    pub(crate) fn insert_char_and_search(&mut self, c: char) {
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
                let target = self.current_dir.join(&path);
                if target.is_dir() {
                    self.change_dir(target);
                    self.clear_input();
                }
                return;
            }
        }

        self.update_query();
    }

    pub(crate) fn delete_char_and_search(&mut self) {
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

    pub(crate) fn delete_until_whitespace(&mut self) {
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

    pub(crate) fn move_left(&mut self, quantity: usize) {
        self.character_index = self.character_index.saturating_sub(quantity);
    }

    pub(crate) fn move_right(&mut self, quantity: usize) {
        self.character_index = (self.character_index + quantity).min(self.input.len());
    }

    pub(crate) fn clear_input(&mut self) {
        self.input = String::new();
        self.character_index = 0;
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }
}
