use crossterm::event::{KeyCode, KeyEvent};
use rand::{random_range, seq::SliceRandom};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    config::config,
    piece::{PIECE_LAYOUTS, Piece},
    screen::{
        AppScreen::{self, Game, Options, Quit},
        game::GameScreen,
        options::OptionsScreen,
    },
    state::State,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum MenuChoice {
    #[default]
    Play,
    Options,
    Quit,
}

impl MenuChoice {
    fn previous(self) -> Self {
        match self {
            Self::Play => Self::Quit,
            Self::Options => Self::Play,
            Self::Quit => Self::Options,
        }
    }
    fn next(self) -> Self {
        match self {
            Self::Play => Self::Options,
            Self::Options => Self::Quit,
            Self::Quit => Self::Play,
        }
    }
}

pub struct TitleScreen {
    selected: MenuChoice,
    activated: Option<MenuChoice>,
    piece_widgets: Vec<Paragraph<'static>>,
}

impl Default for TitleScreen {
    fn default() -> Self {
        let mut piece_widgets = (0..PIECE_LAYOUTS.len())
            .map(|id| {
                let mut piece = Piece::from_id(id);
                for _ in 0..random_range(0..2) * 2 {
                    piece.rotate();
                }
                piece.as_widget()
            })
            .collect::<Vec<Paragraph<'static>>>();
        piece_widgets.shuffle(&mut rand::rng());
        piece_widgets.truncate(6);

        Self {
            selected: MenuChoice::default(),
            activated: None,
            piece_widgets,
        }
    }
}

impl TitleScreen {
    pub fn draw(&self, _state: &mut State, frame: &mut Frame) {
        let config = config();
        let (scale_x, scale_y) = (config.scale_x as u16, config.scale_y as u16);
        drop(config);
        let [content] = Layout::vertical([Constraint::Length(16 + 3 * scale_y)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [body_area] = Layout::vertical([Constraint::Length(30)])
            .flex(Flex::Center)
            .areas(content);

        let [left_pieces, body, right_pieces] = Layout::horizontal([
            Constraint::Length(5 * scale_x),
            Constraint::Length(57),
            Constraint::Length(5 * scale_x),
        ])
        .spacing(4)
        .flex(Flex::Center)
        .areas(body_area);

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
        let [title_area, body_content] =
            Layout::vertical([Constraint::Length(5), Constraint::Length(10)])
                .spacing(1)
                .flex(Flex::Center)
                .areas(body);

        frame.render_widget(title, title_area);

        let [menu_area, controls] =
            Layout::horizontal([Constraint::Length(20), Constraint::Length(30)])
                .spacing(4)
                .flex(Flex::Center)
                .areas(body_content);

        let [play_area, options_area, quit_area] = Layout::vertical([Constraint::Length(3); 3])
            .flex(Flex::Center)
            .areas(menu_area);
        for (area, label, choice) in [
            (play_area, "Play: p", MenuChoice::Play),
            (options_area, "Options: o", MenuChoice::Options),
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

        let half_len = self.piece_widgets.len() / 2;
        let vert_left_pieces: [Rect; 3] =
            Layout::vertical([Constraint::Length(4 * scale_x)].repeat(half_len))
                .flex(Flex::Center)
                .areas(left_pieces);

        let vert_right_pieces: [Rect; 3] =
            Layout::vertical([Constraint::Length(4 * scale_x)].repeat(half_len))
                .flex(Flex::Center)
                .areas(right_pieces);

        for (i, widget) in self.piece_widgets[0..half_len].iter().enumerate() {
            frame.render_widget(widget, vert_left_pieces[i]);
        }

        for (i, widget) in self.piece_widgets[half_len..].iter().enumerate() {
            frame.render_widget(widget, vert_right_pieces[i]);
        }

        let [controls_area] = Layout::vertical([Constraint::Length(8)])
            .flex(Flex::Center)
            .areas(controls);
        let controls_block = Block::bordered()
            .title("Controls")
            .title_alignment(HorizontalAlignment::Center);
        let lines = [
            "A / ←: Move left",
            "D / →: Move right",
            "W / ↑: Rotate",
            "S / ↓: Soft drop",
            "Space: Hard drop",
            "1/2/3: Use powerups",
        ]
        .map(|s| Line::from(vec![Span::from(s)]));
        let controls_para = Paragraph::new(lines.to_vec()).centered();
        frame.render_widget(controls_para, controls_block.inner(controls_area));
        frame.render_widget(controls_block, controls_area);
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = self.selected.previous(),
            KeyCode::Down | KeyCode::Char('s') => self.selected = self.selected.next(),
            KeyCode::Enter => self.activated = Some(self.selected),
            KeyCode::Char('p') => self.activated = Some(MenuChoice::Play),
            KeyCode::Char('o') => self.activated = Some(MenuChoice::Options),
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
            Some(MenuChoice::Options) => Some(Options(OptionsScreen::default())),
            None => None,
        }
    }
}
