mod game;
mod lose;
mod options;
mod title;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};
use std::time::Duration;

use crate::{
    config::config,
    screen::{game::GameScreen, lose::LoseScreen, options::OptionsScreen, title::TitleScreen},
    state::State,
};

const MINIMUM_FIXED_WIDTH: u32 = 64;
const MINIMUM_HEIGHT: u32 = 24;

pub(crate) fn format_elapsed(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
}

fn required_terminal_size() -> (u32, u32) {
    let config = config();
    required_terminal_size_for(
        config.board_width,
        config.board_height,
        config.scale_x,
        config.scale_y,
    )
}

fn required_terminal_size_for(
    board_width: u16,
    board_height: u16,
    scale_x: u16,
    scale_y: u16,
) -> (u32, u32) {
    (
        MINIMUM_FIXED_WIDTH + u32::from(board_width) * u32::from(scale_x),
        MINIMUM_HEIGHT.max(u32::from(board_height) * u32::from(scale_y) + 2),
    )
}

fn terminal_too_small(area: Rect) -> bool {
    let (required_width, required_height) = required_terminal_size();
    u32::from(area.width) < required_width || u32::from(area.height) < required_height
}

fn render_size_warning(frame: &mut Frame) {
    let [area] = Layout::vertical([Constraint::Length(4)])
        .flex(Flex::End)
        .areas(frame.area());
    let warning = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Window too small! The tetris board will be partially cut off from the screen!",
            Style::default().fg(Color::LightRed).bold(),
        )]),
        Line::from("Make your terminal window bigger or lower the scale in Options."),
    ])
    .centered()
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(Color::Red))
    .block(Block::bordered().border_type(BorderType::Double));
    frame.render_widget(Clear, area);
    frame.render_widget(warning, area);
}

pub enum AppScreen {
    Game(GameScreen),
    Title(TitleScreen),
    Options(OptionsScreen),
    Lose(LoseScreen),
    Quit,
}

impl AppScreen {
    pub fn init(&mut self, state: &mut State) {
        match self {
            Self::Game(screen) => screen.init(state),
            Self::Title(screen) => screen.init(),
            Self::Lose(screen) => screen.init(state),
            Self::Options(_) | Self::Quit => {}
        }
    }
    pub fn draw(&mut self, state: &mut State, frame: &mut Frame) {
        match self {
            Self::Game(screen) => screen.draw(state, frame),
            Self::Title(screen) => screen.draw(state, frame),
            Self::Lose(screen) => screen.draw(state, frame),
            Self::Options(screen) => screen.draw(state, frame),
            Self::Quit => {}
        }
    }
    pub fn update(&mut self, state: &mut State) {
        let next_screen = match self {
            Self::Game(screen) => screen.update(state),
            Self::Title(screen) => screen.update(state),
            Self::Lose(screen) => screen.update(state),
            Self::Options(screen) => screen.update(state),
            Self::Quit => None,
        };

        if let Some(mut next_screen_inst) = next_screen {
            next_screen_inst.init(state);
            *self = next_screen_inst;
        }
    }
    pub fn handle_keypress(&mut self, state: &mut State, key: &KeyEvent) {
        match self {
            Self::Game(screen) => screen.handle_keypress(state, key),
            Self::Title(screen) => screen.handle_keypress(state, key),
            Self::Lose(screen) => screen.handle_keypress(state, key),
            Self::Options(screen) => screen.handle_keypress(state, key),
            Self::Quit => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    pub fn captures_text_input(&self) -> bool {
        matches!(self, Self::Lose(screen) if screen.captures_text_input())
    }
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::Title(TitleScreen::default())
    }
}
