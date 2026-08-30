use crossterm::event::{KeyCode, KeyEvent};
use derivative::Derivative;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    config::{config, config_mut, default_config},
    screen::{AppScreen, render_size_warning, terminal_too_small, title::TitleScreen},
    state::State,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum OptionChoice {
    #[default]
    ScaleX,
    ScaleY,
    BoardWidth,
    BoardHeight,
    StartLevel,
    Back,
}

impl OptionChoice {
    fn previous(self) -> Self {
        match self {
            Self::ScaleX => Self::Back,
            Self::ScaleY => Self::ScaleX,
            Self::BoardWidth => Self::ScaleY,
            Self::BoardHeight => Self::BoardWidth,
            Self::StartLevel => Self::BoardHeight,
            Self::Back => Self::StartLevel,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::ScaleX => Self::ScaleY,
            Self::ScaleY => Self::BoardWidth,
            Self::BoardWidth => Self::BoardHeight,
            Self::BoardHeight => Self::StartLevel,
            Self::StartLevel => Self::Back,
            Self::Back => Self::ScaleX,
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct OptionsScreen {
    selected: OptionChoice,
    go_back: bool,

    #[derivative(Default(value = "build_preview_state()"))]
    test_state: State,
}

impl OptionsScreen {
    pub fn draw(&mut self, _state: &mut State, frame: &mut Frame) {
        let config = config();
        let [content] = Layout::vertical([Constraint::Fill(1)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [body, preview_horiz] = Layout::horizontal([
            Constraint::Length(42),
            Constraint::Length(config.board_width * config.scale_x + 2),
        ])
        .flex(Flex::Center)
        .spacing(8)
        .areas(content);
        let [preview] =
            Layout::vertical([Constraint::Length(config.board_height * config.scale_y + 2)])
                .flex(Flex::Center)
                .areas(preview_horiz);
        let [title, desc, buttons] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(24),
        ])
        .spacing(1)
        .flex(Flex::Center)
        .areas(body);
        frame.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Sextant)
                .centered()
                .style(Style::default().add_modifier(Modifier::BOLD))
                .lines(vec![Line::from(vec![Span::styled(
                    "Options",
                    Style::default().fg(Color::Yellow),
                )])])
                .build(),
            title,
        );
        frame.render_widget(
            Paragraph::new(vec![Line::from("←→ to change, r to reset to default")])
                .alignment(HorizontalAlignment::Center),
            desc,
        );

        let areas = Layout::vertical([Constraint::Length(3); 7])
            .flex(Flex::Center)
            .split(buttons);
        let default_config = default_config();
        let choices = [
            (
                "Horizontal scale",
                config.scale_x.to_string(),
                default_config.scale_x.to_string(),
                OptionChoice::ScaleX,
            ),
            (
                "Vertical scale",
                config.scale_y.to_string(),
                default_config.scale_y.to_string(),
                OptionChoice::ScaleY,
            ),
            (
                "Board width",
                config.board_width.to_string(),
                default_config.board_width.to_string(),
                OptionChoice::BoardWidth,
            ),
            (
                "Board height",
                config.board_height.to_string(),
                default_config.board_height.to_string(),
                OptionChoice::BoardHeight,
            ),
            (
                "Starting level",
                config.start_level.to_string(),
                default_config.start_level.to_string(),
                OptionChoice::StartLevel,
            ),
            (
                "",
                "Back (Esc)".to_string(),
                "".to_string(),
                OptionChoice::Back,
            ),
        ];

        for (index, (label, value, default_value, choice)) in choices.into_iter().enumerate() {
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
                .title(format!(
                    "{}{}",
                    label,
                    if value != default_value && !default_value.is_empty() {
                        format!(" ({})", default_value)
                    } else {
                        "".to_string()
                    }
                ))
                .title_alignment(HorizontalAlignment::Center)
                .border_type(if selected {
                    BorderType::Double
                } else {
                    BorderType::Plain
                })
                .border_style(style);
            let text = if choice == OptionChoice::Back {
                value
            } else {
                format!("←  {value}  →",)
            };
            frame.render_widget(
                Paragraph::new(text).centered().style(style).block(block),
                areas[index + (index == areas.len() - 2) as usize],
            );
        }

        if config.board_width != self.test_state.tiles.len() as u16
            || config.board_height != self.test_state.tiles[0].len() as u16
        {
            self.test_state = build_preview_state();
        }
        let game_area = self.test_state.construct_field();
        frame.render_widget(game_area, preview);

        if terminal_too_small(frame.area()) {
            render_size_warning(frame);
        }
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = self.selected.previous(),
            KeyCode::Down | KeyCode::Char('s') => self.selected = self.selected.next(),
            KeyCode::Left | KeyCode::Char('a') => self.adjust(-1),
            KeyCode::Right | KeyCode::Char('d') => self.adjust(1),
            KeyCode::Char('r') => self.reset(),
            KeyCode::Enter if self.selected == OptionChoice::Back => self.go_back = true,
            KeyCode::Esc => self.go_back = true,
            _ => {}
        }
    }

    fn adjust(&self, direction: i8) {
        if self.selected == OptionChoice::Back {
            return;
        }
        let mut config = config_mut();
        let (value, min, max) = match self.selected {
            OptionChoice::ScaleX => (&mut config.scale_x, 1, 8),
            OptionChoice::ScaleY => (&mut config.scale_y, 1, 5),
            OptionChoice::BoardWidth => (&mut config.board_width, 4, 50),
            OptionChoice::BoardHeight => (&mut config.board_height, 4, 50),
            OptionChoice::StartLevel => (&mut config.start_level, 1, 30),
            OptionChoice::Back => return,
        };
        *value = (*value as i16 + direction as i16).clamp(min, max) as u16;
    }

    fn reset(&self) {
        if self.selected == OptionChoice::Back {
            return;
        }
        let mut config = config_mut();
        let default = default_config();
        match self.selected {
            OptionChoice::ScaleX => config.scale_x = default.scale_x,
            OptionChoice::ScaleY => config.scale_y = default.scale_y,
            OptionChoice::BoardWidth => config.board_width = default.board_width,
            OptionChoice::BoardHeight => config.board_height = default.board_height,
            OptionChoice::StartLevel => config.start_level = default.start_level,
            OptionChoice::Back => (),
        };
    }

    pub fn update(&self, _state: &mut State) -> Option<AppScreen> {
        self.go_back.then(|| {
            config().save();
            AppScreen::Title(TitleScreen::default())
        })
    }
}

fn place_if_valid(state: &mut State, x: usize, y: usize, col: Color) {
    if x >= state.tiles.len() || y >= state.tiles[0].len() {
        return;
    }

    state.tiles[x][y] = col;
}

fn build_preview_state() -> State {
    let mut state = State::default();
    let pieces = [
        ([(4_usize, 0), (5, 0), (6, 0), (7, 0)], Color::Blue),
        ([(6, 1), (7, 1), (7, 2), (8, 2)], Color::Magenta),
        ([(8, 0), (9, 0), (8, 1), (9, 1)], Color::Yellow),
    ];
    for (pos, col) in pieces {
        for (x, y) in pos {
            place_if_valid(&mut state, x, y, col);
        }
    }
    state.piece().nudge(0, -2);
    state
}
