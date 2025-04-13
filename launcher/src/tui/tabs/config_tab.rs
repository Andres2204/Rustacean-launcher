use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Rect, Layout, Constraint, Alignment};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListDirection, Widget};
use crate::tui::app::Tab;
use crate::tui::theme::THEME;


// TODO: Change to https://ratatui.rs/templates/component/project-structure/

#[derive(Debug, Clone)]
pub struct ConfigTab {
    config: Vec<ConfigItem>,
    selected_config: usize,
}
#[derive(Debug, Default, Clone)]
pub struct ConfigItem {
    config_name: String,
    config_kind: ConfigKind,
}

#[derive(Debug, Clone)]
pub enum ConfigKind {
    List((usize, Vec<String>)),
    Active(bool),
    Value(((i32, i32), i32)),
}

impl Default for ConfigKind {
    fn default() -> ConfigKind {
        ConfigKind::List(
            (0, Vec::new())
        )
    }
}

impl Default for ConfigTab {
    fn default() -> Self {
        let config = vec![ConfigItem {
            config_name: "Config 1".to_owned(),
            config_kind: ConfigKind::Value(((1, 0), 0)),
        }, ConfigItem {
            config_name: "Config 2".to_owned(),
            config_kind: ConfigKind::List((0, vec!["opt1".to_owned(), "opt2".to_owned()])),
        }, ConfigItem {
            config_name: "Config 3".to_owned(),
            config_kind: ConfigKind::Active(false),
        }];

        ConfigTab { config, selected_config: 0 }
    }
}

impl Widget for ConfigTab {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let chunks = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);
        let [sidebar, config_content] = chunks.areas(area);

        self.render_sidebar(sidebar, buf);
        self.render_config_content(config_content, buf)
    }
}

impl Tab for ConfigTab {
    fn render_tab(&self, area: Rect, buf: &mut Buffer) {
        self.clone().render(area, buf)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.previous_config_tab(),
            KeyCode::Down => self.next_config_tab(),
            _ => {}
        }
    }
}

impl ConfigTab {
    fn render_sidebar(&self, area: Rect, buf: &mut Buffer) {
        let sidebar_block = Block::default()
            .style(THEME.content)
            .borders(Borders::ALL)
            .title("Config")
            .title_alignment(Alignment::Center);

        let items = self.config.clone().iter()
            .enumerate()
            .map(|(i, &ref e)| {
                if i == self.selected_config {
                    format!("> {}", e.config_name)
                } else {
                    e.config_name.clone()
                }
            })
            .collect::<Vec<String>>();

        let config_list = List::new(items)
            .highlight_style(THEME.tabs_selected)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .direction(ListDirection::TopToBottom)
            .block(sidebar_block);

        config_list.render(area, buf);
        // TODO: Create a tabs for config tab o something like that
    }

    fn render_config_content(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .style(THEME.content)
            .borders(Borders::ALL);

        let items: Vec<Line> = self.config.iter().enumerate().map(|(i, e)| {
            let label = {
                if i == self.selected_config { format!("> {}", e.config_name) }
                else { e.config_name.clone() }
            };

            let value = match &self.config[self.selected_config].config_kind {
                ConfigKind::List(_) => { "List".to_owned() }
                ConfigKind::Active(_) => { "Active".to_owned() }
                ConfigKind::Value(_) => { "Value".to_owned() }
            };

            let theme = {
                if i == self.selected_config { THEME.tabs_selected }
                else { THEME.content }
            };

            Line::from(vec![
                Span::raw(label),
                Span::raw(value),
            ]).style(theme)
        }).collect();

        let config_list = List::new(items)
            .highlight_style(THEME.tabs_selected)
            .highlight_symbol("> ")
            .direction(ListDirection::TopToBottom)
            .block(block);

        config_list.render(area, buf)
    }
}

impl ConfigTab {
    fn save_config(&mut self) {}

    fn get_config(&mut self) {}

    // config tabs
    fn next_config_tab(&mut self) {
        if self.selected_config >= self.config.len() {
            self.selected_config = 0;
        } else {
            self.selected_config += 1;
        }
    }

    fn previous_config_tab(&mut self) {
        if self.selected_config == 0 {
            self.selected_config = self.config.len() - 1;
        } else {
            self.selected_config -= 1;
        }
    }

    // config items
    fn handle_config_item(&mut self, up: bool, value: i32) {
        match &self.config[self.selected_config].config_kind {
            &ConfigKind::List((mut index, ref opts)) => {
                if up {
                    if index < opts.len() { index += 1; }
                } else {
                    if index > 0 { index -= 1; }
                }
            }
            &ConfigKind::Active(mut active) => {
                if active { active = false; } else { active = true; }
            }
            &ConfigKind::Value(((ref min, ref max), mut _val)) => {
                if value > *min && value < *max {
                    _val = value;
                }
            }
        }
    }
}