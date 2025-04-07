use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::Paragraph;
use crate::tui::app::Tab;

#[derive(Debug, Default, Copy, Clone)]
pub struct AboutTab;

impl Widget for AboutTab {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized
    {
        Paragraph::new(format!("AboutTab | {}", "This is the about tab"))
            .render(area, buf);
    }
}

impl Tab for AboutTab {
    fn render_tab(&self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code { 
            _ => {}
        }
    }
}