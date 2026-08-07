use crossterm::event::{KeyCode, KeyEvent};
use rand::RngExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    piece::{PIECE_COLOR, PIECE_LAYOUTS},
    screen::{
        AppScreen::{self, Game, Quit},
        game_screen::GameScreen,
    },
    state::State,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum MenuChoice {
    #[default]
    Play,
    Quit,
}

impl MenuChoice {
    fn toggle(self) -> Self {
        match self {
            Self::Play => Self::Quit,
            Self::Quit => Self::Play,
        }
    }
}

pub struct TitleScreen {
    selected: MenuChoice,
    activated: Option<MenuChoice>,
    piece_id: usize,
}

impl Default for TitleScreen {
    fn default() -> Self {
        Self {
            selected: MenuChoice::default(),
            activated: None,
            piece_id: rand::rng().random_range(0..PIECE_LAYOUTS.len()),
        }
    }
}

impl TitleScreen {
    pub fn draw(&self, _state: &mut State, frame: &mut Frame) {
        let [content] = Layout::vertical([Constraint::Length(13)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [title_area, body_area] =
            Layout::vertical([Constraint::Length(5), Constraint::Length(7)])
                .spacing(1)
                .areas(content);

        let title = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .centered()
            .style(Style::default().add_modifier(Modifier::BOLD))
            .lines(vec![Line::from("Cursed Tetris")])
            .build();
        frame.render_widget(title, title_area);

        let [body] = Layout::horizontal([Constraint::Length(44)])
            .flex(Flex::Center)
            .areas(body_area);
        let [menu_area, piece_area] =
            Layout::horizontal([Constraint::Length(20), Constraint::Length(20)])
                .spacing(4)
                .areas(body);

        let [play_area, quit_area] = Layout::vertical([Constraint::Length(3); 2])
            .flex(Flex::Center)
            .areas(menu_area);
        for (area, label, choice) in [
            (play_area, "Play: p", MenuChoice::Play),
            (quit_area, "Quit: q", MenuChoice::Quit),
        ] {
            let selected = self.selected == choice;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let block = Block::bordered()
                .border_type(if selected {
                    BorderType::Double
                } else {
                    BorderType::Plain
                })
                .border_style(style);
            frame.render_widget(
                Paragraph::new(label).centered().style(style).block(block),
                area,
            );
        }

        let [piece_area] = Layout::vertical([Constraint::Length(4)])
            .flex(Flex::Center)
            .areas(piece_area);
        frame.render_widget(self.piece_widget(), piece_area);
    }

    fn piece_widget(&self) -> Paragraph<'static> {
        let layout = PIECE_LAYOUTS[self.piece_id];
        let min_x = layout.iter().map(|position| position[0]).min().unwrap_or(0);
        let max_x = layout.iter().map(|position| position[0]).max().unwrap_or(0);
        let min_y = layout.iter().map(|position| position[1]).min().unwrap_or(0);
        let max_y = layout.iter().map(|position| position[1]).max().unwrap_or(0);
        let color = PIECE_COLOR[self.piece_id];
        let mut lines = Vec::new();

        for y in (min_y..=max_y).rev() {
            let spans: Vec<Span<'static>> = (min_x..=max_x)
                .map(|x| {
                    let filled = layout.contains(&[x, y]);
                    Span::styled(
                        if filled { "████" } else { "    " },
                        Style::default().fg(color),
                    )
                })
                .collect();
            let line = Line::from(spans).centered();
            lines.push(line.clone());
            lines.push(line);
        }

        Paragraph::new(lines)
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Char('w') | KeyCode::Char('s') => {
                self.selected = self.selected.toggle()
            }
            KeyCode::Enter => self.activated = Some(self.selected),
            KeyCode::Char('p') => self.activated = Some(MenuChoice::Play),
            KeyCode::Char('q') => self.activated = Some(MenuChoice::Quit),
            _ => {}
        }
    }

    pub fn update(&self, state: &mut State) -> Option<AppScreen> {
        match self.activated {
            Some(MenuChoice::Play) => {
                *state = State::default();
                Some(Game(GameScreen::default()))
            }
            Some(MenuChoice::Quit) => Some(Quit),
            None => None,
        }
    }
}
