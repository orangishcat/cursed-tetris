use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Wrap},
};
use std::time::Duration;
use tui_big_text::{BigText, PixelSize};

use crate::{
    config::config,
    leaderboard::{Qualification, USERNAME_MAX_LEN, online, online_mut},
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
    submission: SubmissionState,
}

#[derive(Default)]
enum SubmissionState {
    #[default]
    None,
    Eligible {
        qualification: Qualification,
        input: String,
        error: Option<String>,
    },
    Submitted(usize),
    Skipped,
    Error(String),
}

impl LoseScreen {
    pub fn new(elapsed: Duration) -> Self {
        Self {
            elapsed,
            ..Self::default()
        }
    }

    pub fn init(&mut self, state: &State) {
        if !config().has_default_submission_settings() {
            self.submission = SubmissionState::None;
            return;
        }
        let online = online();
        self.submission = match online.as_ref() {
            Some(online) => match online.qualification(state.score) {
                Ok(Some(qualification)) => SubmissionState::Eligible {
                    qualification,
                    input: String::new(),
                    error: None,
                },
                Ok(None) => SubmissionState::None,
                Err(error) => SubmissionState::Error(error),
            },
            None => SubmissionState::None,
        };
    }

    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        let high_score = config().high_score;
        let has_submission = !matches!(self.submission, SubmissionState::None);
        let content_height = if has_submission { 23 } else { 15 };
        let [content] = Layout::vertical([Constraint::Length(content_height)])
            .flex(Flex::Center)
            .areas(frame.area());
        let mut constraints = vec![
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(3),
        ];
        if has_submission {
            constraints.insert(2, Constraint::Length(7));
        }
        let areas = Layout::vertical(constraints).spacing(1).split(content);
        let banner_area = areas[0];
        let stats_area = areas[1];
        let actions_area = *areas.last().expect("lose screen always has action area");

        if has_submission {
            let [submission_area] = Layout::horizontal([Constraint::Length(48)])
                .flex(Flex::Center)
                .areas(areas[2]);
            self.draw_submission(frame, submission_area);
        }

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

    pub fn handle_keypress(&mut self, state: &mut State, key: &KeyEvent) {
        if matches!(self.submission, SubmissionState::Eligible { .. })
            && self.handle_submission(state.score, key)
        {
            return;
        }
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

    pub fn captures_text_input(&self) -> bool {
        matches!(
            self.submission,
            SubmissionState::Eligible {
                qualification: Qualification { username: None, .. },
                ..
            }
        )
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

    fn handle_submission(&mut self, score: u32, key: &KeyEvent) -> bool {
        if key.code == KeyCode::Esc {
            self.submission = SubmissionState::Skipped;
            return true;
        }

        let SubmissionState::Eligible {
            qualification,
            input,
            error,
        } = &mut self.submission
        else {
            return false;
        };
        if qualification.username.is_none() {
            match key.code {
                KeyCode::Backspace => {
                    input.pop();
                    *error = None;
                    return true;
                }
                KeyCode::Char(character)
                    if character.is_ascii_alphanumeric()
                        || character == '_'
                        || character == '-' =>
                {
                    if input.len() < USERNAME_MAX_LEN {
                        input.push(character);
                        *error = None;
                    }
                    return true;
                }
                _ => {}
            }
        }
        if key.code != KeyCode::Enter {
            return true;
        }

        let username = qualification.username.is_none().then_some(input.clone());
        let mut online = online_mut();
        let Some(online) = online.as_mut() else {
            self.submission = SubmissionState::Error("Leaderboard unavailable.".into());
            return true;
        };
        match online.submit(score, username.as_deref()) {
            Ok(rank) => self.submission = SubmissionState::Submitted(rank),
            Err(message) => {
                if let SubmissionState::Eligible { error, .. } = &mut self.submission {
                    *error = Some(message);
                }
            }
        };
        true
    }

    fn draw_submission(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let (message, style) = match &self.submission {
            SubmissionState::None => (String::new(), Style::default()),
            SubmissionState::Eligible {
                qualification,
                input,
                error,
            } => {
                let prompt = qualification.username.as_ref().map_or_else(
                    || format!("You got a #{} score on the leaderboard! Type a name and press Enter to submit your score, otherwise Esc to skip submission.\nName: {input}_", qualification.rank),
                    |username| {
                        format!(
                            "\nNew #{} score as {username}, submit?\nEnter: submit / Esc: skip",
                            qualification.rank
                        )
                    },
                );
                let message = error
                    .as_ref()
                    .map_or(prompt.clone(), |error| format!("Name: {input}_\n{error}"));
                (
                    message,
                    Style::default().fg(if error.is_some() {
                        Color::LightRed
                    } else {
                        Color::LightYellow
                    }),
                )
            }
            SubmissionState::Submitted(rank) => (
                format!("\nScore submitted at rank #{rank}."),
                Style::default().fg(Color::Green),
            ),
            SubmissionState::Skipped => ("\nScore submission skipped".into(), Style::default()),
            SubmissionState::Error(error) => (
                format!("\nLeaderboard unavailable: {error}"),
                Style::default().fg(Color::LightRed),
            ),
        };
        frame.render_widget(
            Paragraph::new(message)
                .centered()
                .wrap(Wrap { trim: true })
                .style(style)
                .block(
                    Block::bordered()
                        .title("Score submission")
                        .title_alignment(HorizontalAlignment::Center),
                ),
            area,
        );
    }
}
