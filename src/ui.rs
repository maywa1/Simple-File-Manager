use std::path::Path;

use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::app::Modes;

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.mode {
        Modes::FileOpen => { },
        Modes::Search => search_ui(app, frame),
        Modes::Action => action_ui(app, frame),
        Modes::Rename => rename_ui(app, frame),
        Modes::CreateFileOrDir => create_file_or_dir_ui(app, frame),
        Modes::DeleteConfirm => delete_confirm_ui(app, frame),
    }
}


fn search_ui(app: &mut App,frame: &mut Frame){
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]);

    let [help_area, input_area, results_area] = frame.area().layout(&layout);

    let help = if let Some(ref status) = app.status {
        Paragraph::new(Line::from(status.as_str()))
            .style(Style::default().fg(Color::Yellow))
    } else if app.moving {
        let file_name = app
            .action_target
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        Paragraph::new(Line::from(vec![
            format!("Moving {file_name}").into(),
            " | navigate to dir, Enter to move, Esc to cancel".into(),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            "Ctrl+C".into(),
            " quit | type to search".into(),
        ]))
    };

    frame.render_widget(help, help_area);

    let dir_prefix = format!("{} > ", app.current_dir.display());
    let input_content = format!("{dir_prefix}{}", app.input);

    let title = if app.moving { "Move" } else { "Search" };
    let input = Paragraph::new(input_content.as_str())
        .block(Block::bordered().title(title))
        .style(Style::default().fg(Color::LightMagenta));

    frame.render_widget(input, input_area);

    let items: Vec<ListItem> = if app.input.contains('*') {
        app.glob_results
            .iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect()
    } else {
        let snapshot = app.nucleo.snapshot();
        snapshot
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
            .collect()
    };

    let results = List::new(items).block(Block::bordered().title("Files"));

    frame.render_widget(results, results_area);

    let prefix_len = dir_prefix.len() as u16;
    frame.set_cursor_position(Position::new(
        input_area.x + prefix_len + app.character_index as u16 + 1,
        input_area.y + 1,
    ));
}

fn create_file_or_dir_ui(app: &mut App, frame: &mut Frame) {
    let file_name = app
        .action_target
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let title = format!(" {file_name} ");
    let actions = vec![
        "d - create directory",
        "f - create file",
        "Esc - back",
    ];

    let width = actions.iter().map(|a| a.len()).max().unwrap_or(0).max(20) + 4;
    let height = actions.len() + 2;
    let area = frame.area().centered(
        Constraint::Length(width as u16),
        Constraint::Length(height as u16),
    );

    let text = actions
        .iter()
        .map(|a| Line::from(*a))
        .collect::<Vec<_>>();

    let popup = Paragraph::new(text)
        .block(Block::bordered().title(title.as_str()))
        .style(Style::default().fg(Color::LightMagenta));

    frame.render_widget(popup, area);
}


fn action_ui(app: &mut App, frame: &mut Frame) {
    let file_name = app
        .action_target
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let title = format!(" {file_name} ");
    let actions = vec![
        "o - open",
        "r - rename",
        "d - delete",
        "y - copy path",
        "m - move",
        "Esc - back",
    ];

    let width = actions.iter().map(|a| a.len()).max().unwrap_or(0).max(20) + 4;
    let height = actions.len() + 2;
    let area = frame.area().centered(
        Constraint::Length(width as u16),
        Constraint::Length(height as u16),
    );

    let text = actions
        .iter()
        .map(|a| Line::from(*a))
        .collect::<Vec<_>>();

    let popup = Paragraph::new(text)
        .block(Block::bordered().title(title.as_str()))
        .style(Style::default().fg(Color::LightMagenta));

    frame.render_widget(popup, area);
}
fn rename_ui(app: &mut App, frame: &mut Frame) {
    let old_name = app
        .action_target
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let title = format!(" Rename {old_name} to:");
    let input_content = format!("  {}", app.input);

    let input = Paragraph::new(input_content.as_str())
        .block(Block::bordered().title(title.as_str()))
        .style(Style::default().fg(Color::LightMagenta));

    let width = 60.max(title.len() as u16);
    let area = frame.area().centered(
        Constraint::Length(width),
        Constraint::Length(3),
    );

    frame.render_widget(input, area);

    frame.set_cursor_position(Position::new(
        area.x + 2 + app.character_index as u16,
        area.y + 1,
    ));
}

fn delete_confirm_ui(app: &mut App, frame: &mut Frame) {
    let file_name = app
        .action_target
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let msg = format!(" Delete {file_name}? (y/N) ");
    let width = msg.len() as u16 + 2;

    let popup = Paragraph::new("")
        .block(Block::bordered().title(msg))
        .style(Style::default().fg(Color::LightMagenta));

    let area = frame.area().centered(
        Constraint::Length(width),
        Constraint::Length(3),
    );

    frame.render_widget(popup, area);
}
