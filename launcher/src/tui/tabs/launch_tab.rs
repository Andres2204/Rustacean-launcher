use crate::core::downloader::downloader::DownloaderTracking;
use crate::core::launcher::launcher::LauncherBuilder;
use crate::core::users::UserBuilder;
use crate::core::versions::version::Version;
use crate::core::versions::version_manager::VersionManager;
use crate::tui::app::Tab;
use crate::tui::tabs::launch_tab::LaunchTabState::{DOWNLOADING, NORMAL};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, List, Paragraph};
use std::sync::Arc;
use std::thread;
use tokio::sync::{Mutex};
use tui_widget_list::{ListBuilder, ListState, ListView};

#[derive(Clone, Default)]
pub struct LaunchTab {
    cached_versions: Vec<Box<dyn Version>>,
    selected_version: Option<Box<dyn Version>>,
    selected_index: usize,
    download_progress: Option<Arc<Mutex<DownloaderTracking>>>,
    state: LaunchTabState,
    list_state: ListState,
}

#[derive(Clone, Default)]
enum LaunchTabState {
    #[default]
    NORMAL,
    DOWNLOADING,
    VERIFYING,
    LAUNCHING,
}

impl LaunchTab {
    pub fn new() -> Self {
        Self {
            cached_versions: {
                let fetched = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(VersionManager::fetch_versions())
                });
                fetched.unwrap()
            },
            selected_version: None,
            selected_index: 0,
            download_progress: None,
            state: LaunchTabState::default(),
            list_state: ListState::default(),
        }
    }
}

impl Widget for LaunchTab {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]);
        let [version_log, progress_bar] = layout.areas(area);

        let [version_selectioner, log_screen] =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
                .areas(version_log);

        self.render_progress(progress_bar, buf);
        self.render_versions_selectioner(version_selectioner, buf);
        self.render_log_screen(log_screen, buf);
    }
}

impl Tab for LaunchTab {
    fn render_tab(&self, area: Rect, buf: &mut Buffer) {
        self.clone().render(area, buf);
    }
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.list_state.selected = Some(self.selected_index);
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.cached_versions.len().saturating_sub(1) {
                    self.selected_index += 1;
                    self.list_state.selected = Some(self.selected_index);
                }
            }
            KeyCode::Enter => {
                self.selected_version = self.cached_versions.get(self.selected_index).cloned();
            }
            KeyCode::Char('a') => { // TODO: Set state DOWNLOADING
                match &self.selected_version {
                    Some(version) => {
                        let progress = match &self.download_progress {
                            Some(p) => { p.clone() }
                            None => {
                                let p = Arc::new(Mutex::new(DownloaderTracking::default()));
                                self.download_progress = Some(p.clone());
                                p.clone()
                            }
                        };
                        let v = version.clone();
                        log::info!("Starting download");
                        tokio::spawn(async move {
                            log::info!("Living in parallel");
                            VersionManager::download_version(v, progress).await.expect("[LaunchTab] Downloading error");
                        });
                    }
                    None => {log::info!("not selected")} // TODO: not version selected advice
                }
            } // download
            KeyCode::Char('d') => {} // verify
            KeyCode::Char(' ') => {
                self.state = LaunchTabState::LAUNCHING;

                if self.selected_version.is_some() {
                    let ml = LauncherBuilder::new()
                        .version(self.selected_version.clone().unwrap())
                        .user(UserBuilder::default_boxed())
                        .build();
                    ml.launch_minecraft().unwrap()
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListItem {
    text: String,
    style: Style,
}

impl ListItem {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }
}

impl Widget for ListItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::from(self.text).style(self.style).render(area, buf);
    }
}

// renders
impl LaunchTab {
    fn render_versions_selectioner(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_name: &str = if let Some(version) = &self.selected_version {
            &version.name()
        } else {
            "?"
        };

        let versions: Vec<String> = self
            .cached_versions
            .clone()
            .into_iter()
            .map(|v| v.name())
            .collect();

        let builder = ListBuilder::new(|context| {
            let mut item = ListItem::new(&versions[context.index]);

            // Alternating styles
            if context.index % 2 == 0 {
                item.style = Style::default().bg(Color::Rgb(28, 28, 32));
            } else {
                item.style = Style::default().bg(Color::Rgb(0, 0, 0));
            }

            // tint installed versions
            let version = self.cached_versions.get(context.index).cloned();
            if let Some(version) = version {
                if VersionManager::is_installed(version) {
                    item.style = Style::default().bg(Color::Rgb(34, 28, 132));
                }
            }

            // Style the selected element
            if context.is_selected {
                item.style = Style::default()
                    .bg(Color::Rgb(255, 153, 0))
                    .fg(Color::Rgb(28, 28, 32));
            };

            // Return the size of the widget along the main axis.
            let main_axis_size = 1;
            (item, main_axis_size)
        });

        let item_count = versions.len();
        let list = ListView::new(builder, item_count);
        let state = &mut self.list_state;

        list.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("Version: {}", selected_name))
                .title_alignment(Alignment::Center),
        )
        .render(area, buf, state);
    }

    fn render_log_screen(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Logs ")
            .title_alignment(Alignment::Center);

        let mut lines: Vec<Line> = if let Some(progress) = self.download_progress.clone() {
            let progress = progress.try_lock();
            let units_len = match &progress {
                Ok(u) => {u.units().clone().iter().len()}
                _ => {69}
            };
            let mut l: Vec<Line> = vec![
                Line::raw(format!("Units: {}", units_len)),
                Line::raw(format!("{:?}", progress)),
            ];

            match progress {
                Ok(u) => {
                    u.units().iter()
                        .map(|p| {
                            let file_progress = p.read();
                            match file_progress {
                                Ok(progress) => l.push(Line::raw(format!("progress {:?}", progress.name()))),
                                _ => {l.push(Line::raw("progress not available"))},
                            }
                        })
                        .collect()
                },
                _ => { l.push(Line::raw("not downloading")) }
            }
            l
        } else {
            vec![Line::raw("not downloading")]
        };

        // Renderizar la lista
        lines.push(Line::raw("¡DOWNLOADING!"));
        Widget::render(List::new(lines).block(block), area, buf);
    }

    fn render_progress(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        Paragraph::new("This is the launch tab")
            .alignment(Alignment::Center)
            .block(block)
            .render(area, buf);
    }
}
