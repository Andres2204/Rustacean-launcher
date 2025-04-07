use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Widget;
use ratatui::widgets::Paragraph;
use crate::tui::app::Tab;

#[derive(Debug, Default, Clone)]
pub struct LaunchTab {
    selected_version: String,
}
impl Widget for LaunchTab {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        Paragraph::new("This is the launch tab")
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

impl Tab for LaunchTab {
    fn render_tab(&self, area: Rect, buf: &mut Buffer) {
        self.clone().render(area, buf);
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let key2 = key.code.to_string();
        println!("METHOD no implemented handle_key: {:?}", key2);
    }
}