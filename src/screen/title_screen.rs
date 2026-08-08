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
    piece::{PIECE_LAYOUTS, Piece},
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
    piece_widget: Paragraph<'static>,
}

impl Default for TitleScreen {
    fn default() -> Self {
        Self {
            selected: MenuChoice::default(),
            activated: None,
            piece_widget: Piece::from_id(rand::rng().random_range(0..PIECE_LAYOUTS.len()))
                .as_widget(),
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
            .lines(vec![Line::from(vec![
                Span::styled("Cursed", Style::default().fg(Color::LightRed)),
                Span::raw(" "),
                Span::styled("Tetris", Style::default().fg(Color::Yellow)),
            ])])
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
        frame.render_widget(&self.piece_widget, piece_area);
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
