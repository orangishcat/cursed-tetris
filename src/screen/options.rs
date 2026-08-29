use std::{cmp::max, fmt::format};

use crossterm::event::{KeyCode, KeyEvent};
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
    screen::{AppScreen, title::TitleScreen},
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

#[derive(Default)]
pub struct OptionsScreen {
    selected: OptionChoice,
    go_back: bool,
}

impl OptionsScreen {
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        let config = config();
        let [content] = Layout::vertical([Constraint::Length(max(
            32,
            config.board_height * config.scale_y,
        ))])
        .flex(Flex::Center)
        .areas(frame.area());
        let [body, preview] = Layout::horizontal([
            Constraint::Length(42),
            Constraint::Length((config.board_width * config.scale_x) as u16),
        ])
        .flex(Flex::Center)
        .spacing(4)
        .areas(content);
        let [title, buttons] = Layout::vertical([Constraint::Length(5), Constraint::Length(24)])
            .flex(Flex::Center)
            .areas(body);
        frame.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Quadrant)
                .centered()
                .style(Style::default().add_modifier(Modifier::BOLD))
                .lines(vec![Line::from(vec![Span::styled(
                    "Options",
                    Style::default().fg(Color::Yellow),
                )])])
                .build(),
            title,
        );

        let areas = Layout::vertical([Constraint::Length(3); 6])
            .flex(Flex::Center)
            .spacing(1)
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
            ("", "Back".to_string(), "".to_string(), OptionChoice::Back),
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
                areas[index],
            );
        }
        let game_area = state.construct_field();
        frame.render_widget(game_area, preview);
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = self.selected.previous(),
            KeyCode::Down | KeyCode::Char('s') => self.selected = self.selected.next(),
            KeyCode::Left | KeyCode::Char('a') => self.adjust(-1),
            KeyCode::Right | KeyCode::Char('d') => self.adjust(1),
            KeyCode::Enter if self.selected == OptionChoice::Back => self.go_back = true,
            KeyCode::Char('b') | KeyCode::Char('t') => self.go_back = true,
            _ => {}
        }
    }

    fn adjust(&self, direction: i8) {
        if self.selected == OptionChoice::Back {
            return;
        }
        let mut config = config_mut();
        let (value, min, max) = match self.selected {
            OptionChoice::ScaleX => (&mut config.scale_x, 1, 5),
            OptionChoice::ScaleY => (&mut config.scale_y, 1, 5),
            OptionChoice::BoardWidth => (&mut config.board_width, 4, 50),
            OptionChoice::BoardHeight => (&mut config.board_height, 4, 50),
            OptionChoice::StartLevel => (&mut config.start_level, 1, 30),
            OptionChoice::Back => return,
        };
        *value = (*value as i16 + direction as i16).clamp(min, max) as u16;
    }

    pub fn update(&self, _state: &mut State) -> Option<AppScreen> {
        self.go_back.then(|| {
            config().save();
            AppScreen::Title(TitleScreen::default())
        })
    }
}
