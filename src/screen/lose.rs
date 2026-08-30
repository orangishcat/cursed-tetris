use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};
use std::time::Duration;
use tui_big_text::{BigText, PixelSize};

use crate::{
    config::config,
    screen::{
        AppScreen::{self, Game, Quit, Title},
        format_elapsed,
        game::GameScreen,
        title::TitleScreen,
    },
    state::State,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum MenuChoice {
    #[default]
    Restart,
    Title,
    Quit,
}

impl MenuChoice {
    fn previous(self) -> Self {
        match self {
            Self::Restart => Self::Quit,
            Self::Title => Self::Restart,
            Self::Quit => Self::Title,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Restart => Self::Title,
            Self::Title => Self::Quit,
            Self::Quit => Self::Restart,
        }
    }
}

#[derive(Default)]
pub struct LoseScreen {
    selected: MenuChoice,
    activated: Option<MenuChoice>,
    elapsed: Duration,
}

impl LoseScreen {
    pub fn new(elapsed: Duration) -> Self {
        Self {
            elapsed,
            ..Self::default()
        }
    }

    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        let high_score = config().high_score;
        let [content] = Layout::vertical([Constraint::Length(15)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [banner_area, stats_area, actions_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(3),
        ])
        .spacing(1)
        .areas(content);

        let game_over = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .centered()
            .style(Style::default().add_modifier(Modifier::BOLD).light_red())
            .lines(vec![Line::from("Game Over!")])
            .build();
        frame.render_widget(game_over, banner_area);

        let [first_row, second_row] =
            Layout::vertical([Constraint::Length(3); 2]).areas(stats_area);
        let [score_area, high_score_area] = Layout::horizontal([Constraint::Length(24); 2])
            .spacing(1)
            .flex(Flex::Center)
            .areas(first_row);
        let [level_pieces_area, time_area] = Layout::horizontal([Constraint::Length(24); 2])
            .spacing(1)
            .flex(Flex::Center)
            .areas(second_row);
        for (area, title, value) in [
            (score_area, "Score", state.score.to_string()),
            (high_score_area, "High Score", high_score.to_string()),
            (
                level_pieces_area,
                "Level / Pieces",
                format!("{} / {}", state.level, state.placed_pieces),
            ),
            (time_area, "Time", format_elapsed(self.elapsed)),
        ] {
            frame.render_widget(
                Paragraph::new(value).centered().block(
                    Block::bordered()
                        .title(title)
                        .title_alignment(HorizontalAlignment::Center),
                ),
                area,
            );
        }

        let [actions] = Layout::horizontal([Constraint::Length(54)])
            .flex(Flex::Center)
            .areas(actions_area);
        let action_areas = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(actions);
        for (index, (label, choice)) in [
            ("Restart: r", MenuChoice::Restart),
            ("Title: t", MenuChoice::Title),
            ("Quit: q", MenuChoice::Quit),
        ]
        .into_iter()
        .enumerate()
        {
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
                action_areas[index],
            );
        }
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('a') => self.selected = self.selected.previous(),
            KeyCode::Right | KeyCode::Char('d') => self.selected = self.selected.next(),
            KeyCode::Enter => self.activated = Some(self.selected),
            KeyCode::Char('r') => self.activated = Some(MenuChoice::Restart),
            KeyCode::Char('t') => self.activated = Some(MenuChoice::Title),
            KeyCode::Char('q') => self.activated = Some(MenuChoice::Quit),
            _ => {}
        }
    }

    pub fn update(&self, state: &mut State) -> Option<AppScreen> {
        match self.activated {
            Some(MenuChoice::Restart) => {
                *state = State::default();
                Some(Game(GameScreen::default()))
            }
            Some(MenuChoice::Title) => Some(Title(TitleScreen::default())),
            Some(MenuChoice::Quit) => Some(Quit),
            None => None,
        }
    }
}
