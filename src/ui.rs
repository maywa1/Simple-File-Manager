use std::path::Path;

use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]);

    let [help_area, input_area, results_area] = frame.area().layout(&layout);

    let help = Paragraph::new(Line::from(vec![
        "Ctrl+C".into(),
        " quit | type to search".into(),
    ]));

    frame.render_widget(help, help_area);

    let dir_prefix = format!("{} > ", app.current_dir.display());
    let input_content = format!("{dir_prefix}{}", app.input);

    let input = Paragraph::new(input_content.as_str())
        .block(Block::bordered().title("Search"))
        .style(Style::default().fg(Color::LightMagenta));

    frame.render_widget(input, input_area);

    let snapshot = app.nucleo.snapshot();

    let items: Vec<ListItem> = snapshot
        .matched_items(..)
        .take(30)
        .map(|item| {
            let path = Path::new(&item.data);
            let display = path
                .strip_prefix(&app.current_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            ListItem::new(display)
        })
        .collect();

    let results = List::new(items).block(Block::bordered().title("Files"));

    frame.render_widget(results, results_area);

    let prefix_len = dir_prefix.len() as u16;
    frame.set_cursor_position(Position::new(
        input_area.x + prefix_len + app.character_index as u16 + 1,
        input_area.y + 1,
    ));
}
