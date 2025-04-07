use crate::tui::theme::THEME;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{
    buffer::Buffer,
    crossterm::event,
    crossterm::event::{
        Event,
        KeyEvent,
        KeyEventKind
    },
    layout::{
        Constraint,
        Layout,
        Rect
    }
    ,
    text::Span,
    widgets::{
        Block,
        Widget,
        Tabs
    },
    Terminal,
    Frame
};
use std::{
    io,
    io::Stdout,
    time::Duration,
};
use ratatui::style::Color;
use ratatui::text::Line;
use crate::tui::tabs;
use tabs::LaunchTab;
use tabs::ConfigTab;
use tabs::AboutTab;
use crate::command::command::Command;
use crate::command::commands::launch::LaunchCommand;

// TODO: Encontrar forma de no clonar el tabwidget
// volviendolo un widget solamente ? 
// nose
pub struct App {
    tabs: Vec<Box<dyn TabWidget>>,
    selected_tab: usize,
    is_running: bool,
}

pub trait Tab {
    fn render_tab(&self, area: Rect, buf: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent);
}

pub trait TabWidget: Tab + Widget {}
impl<T> TabWidget for T where T: Tab + Widget {}

impl App {
    fn new() -> App {
        App { 
            tabs: vec![
                Box::new(LaunchTab::new()),
                Box::new(ConfigTab::default()),
                Box::new(AboutTab::default())
            ], 
            selected_tab: 0, 
            is_running: true 
        }
    }

    pub fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<bool> {
        loop {
            if !self.is_running { break Ok(true); };
            
            terminal
                .draw(|frame| self.draw(frame))
                .expect("[App/run_app] terminal.draw");
            self.handle_events()?;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.prev_tab(),
            KeyCode::Char('e') => self.next_tab(),
            KeyCode::Char('c') => self.is_running = false,
            KeyCode::Char('l') => {
                LaunchCommand.execute();
                self.is_running = false;
            },
            _ => self.tabs[self.selected_tab].handle_key(key)
        }
    }

    fn prev_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = self.tabs.len() - 1;
        } else {
            self.selected_tab -= 1;
        }
    }

    fn next_tab(&mut self) {
        if self.selected_tab == self.tabs.len() - 1 {
            self.selected_tab = 0
        } else {
            self.selected_tab += 1;
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        let vertical_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [title_bar, tab, bottom_bar] = vertical_layout.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        self.render_title_bar(title_bar, buf);
        self.render_selected_tab(tab, buf);
        self.render_bottom_bar(bottom_bar, buf);
    }
}

impl App {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer ) {
        let layout = Layout::horizontal([
            Constraint::Min("Rustacean Launcher".len() as u16),
            Constraint::Length(40), // TODO: adjust tab size
        ]);
        let [title, tabs] = layout.areas(area);

        Span::styled("Rustacean Launcher", THEME.app_title).render(title, buf);
        let tab_titles = vec!["Launch","Config", "About"];
        Tabs::new(tab_titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .select(self.selected_tab)
            .divider("|")
            .padding(" ", " ")
            .render(tabs, buf);
    }

    fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        let tab = &self.tabs[self.selected_tab];
        tab.render_tab(area, buf);
    }
    
    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let keys = [
            ("H/←", "Left"),
            ("L/→", "Right"),
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("D/Del", "Destroy"),
            ("Q/Esc", "Quit"),
        ];
        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect();
        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}
impl Tui {
    pub fn new() -> Self {
        Self { // TODO: FIX
            terminal: Terminal::new(CrosstermBackend::new(io::stdout())).unwrap(),
        }
    }

    pub fn run_tui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        color_eyre::install()?;
        self.init_panic_hook();
        let _ = self.init_tui();

        let mut app = App::new();
        app.run_app(&mut self.terminal).expect("[APP/run_tui] Error running app");

        if let Err(err) = self.restore_tui() {
            eprintln!(
                "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
                err
            );
        }
        Ok(())
    }

    fn init_tui(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("Cant initialize terminal");

        let backend = CrosstermBackend::new(stdout);
        self.terminal = Terminal::new(backend)?;
        Ok(())
    }

    fn restore_tui(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn init_panic_hook(&mut self) {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // TODO: self.restore_tui().expect("Cant restore tui."); // ignore any errors as we are already failing
            disable_raw_mode().unwrap();
            execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture,
            ).unwrap();
            hook(panic_info);
        }));
    }
}


