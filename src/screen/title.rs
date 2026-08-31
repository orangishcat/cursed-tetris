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
    leaderboard::{LEADERBOARD_LIMIT, Online, online, online_mut},
    piece::{PIECE_LAYOUTS, Piece},
    screen::{
        AppScreen::{self, Game, Options, Quit},
        game::GameScreen,
        options::OptionsScreen,
        render_size_warning, terminal_too_small,
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
    play_disabled: bool,
    leaderboard_offset: usize,
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
            play_disabled: false,
            leaderboard_offset: 0,
        }
    }
}

impl TitleScreen {
    pub fn init(&mut self) {
        let mut online = online_mut();
        if let Some(online) = online.as_mut() {
            online.refresh();
            self.clamp_leaderboard_offset(online.entries().len());
        }
    }

    pub fn draw(&mut self, _state: &mut State, frame: &mut Frame) {
        let online = online();
        let online = online.as_ref();
        self.play_disabled = terminal_too_small(frame.area());
        let config = config();
        let (scale_x, scale_y, high_score) = (config.scale_x, config.scale_y, config.high_score);
        drop(config);
        let [content] = Layout::vertical([Constraint::Length(17 + 3 * scale_y)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [body_area] = Layout::vertical([Constraint::Length(30)])
            .flex(Flex::Center)
            .areas(content);

        let right_width = if online.is_some() { 29 } else { 5 * scale_x };
        let [left_pieces, body, right_pieces] = Layout::horizontal([
            Constraint::Length(5 * scale_x),
            Constraint::Length(57),
            Constraint::Length(right_width),
        ])
        .spacing(4)
        .flex(Flex::Center)
        .areas(body_area);

        let title = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .centered()
            .style(Style::default().add_modifier(Modifier::BOLD))
            .lines(vec![Line::from(vec![
                Span::styled("Cursed", Style::default().fg(Color::LightRed)),
                Span::raw(" "),
                Span::styled("Tetris", Style::default().fg(Color::Yellow)),
            ])])
            .build();
        let [title_area, body_content] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(11)])
                .spacing(1)
                .flex(Flex::Center)
                .areas(body);

        frame.render_widget(title, title_area);

        let [menu_area, controls] =
            Layout::horizontal([Constraint::Length(20), Constraint::Length(30)])
                .spacing(4)
                .flex(Flex::Center)
                .areas(body_content);

        let [high_score_area, play_area, options_area, quit_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .flex(Flex::Center)
        .areas(menu_area);
        for (area, label, choice) in [
            (play_area, "Play: p", MenuChoice::Play),
            (options_area, "Options: o", MenuChoice::Options),
            (quit_area, "Quit: q", MenuChoice::Quit),
        ] {
            let selected = self.selected == choice;
            let disabled = choice == MenuChoice::Play && self.play_disabled;
            let style = if disabled {
                Style::default().fg(Color::DarkGray)
            } else if selected {
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
        frame.render_widget(
            Paragraph::new(format!("High score: {high_score}"))
                .centered()
                .style(Style::default().fg(Color::Yellow)),
            high_score_area,
        );

        let half_len = self.piece_widgets.len() / 2;
        let vert_left_pieces: [Rect; 3] =
            Layout::vertical([Constraint::Length(4 * scale_x)].repeat(half_len))
                .flex(Flex::Center)
                .areas(left_pieces);

        for (i, widget) in self.piece_widgets[0..half_len].iter().enumerate() {
            frame.render_widget(widget, vert_left_pieces[i]);
        }

        if let Some(online) = online {
            self.draw_leaderboard(frame, right_pieces, online);
        } else {
            let vert_right_pieces: [Rect; 3] =
                Layout::vertical([Constraint::Length(4 * scale_x)].repeat(half_len))
                    .flex(Flex::Center)
                    .areas(right_pieces);
            for (i, widget) in self.piece_widgets[half_len..].iter().enumerate() {
                frame.render_widget(widget, vert_right_pieces[i]);
            }
        }

        let lines = [
            "A / ←: Move left",
            "D / →: Move right",
            "W / ↑: Rotate",
            "S / ↓: Soft drop",
            "Space: Hard drop",
            "Shift/c: Hold piece",
            "1/2/3: Use powerups",
            "Esc/p: Pause",
            "q/Ctrl+C: Quit",
        ]
        .map(|s| Line::from(vec![Span::from(s)]));
        let [controls_area] = Layout::vertical([Constraint::Length(lines.len() as u16 + 2)])
            .flex(Flex::Center)
            .areas(controls);
        let controls_block = Block::bordered()
            .title("Controls")
            .title_alignment(HorizontalAlignment::Center);
        let controls_para = Paragraph::new(lines.to_vec()).centered();
        frame.render_widget(controls_para, controls_block.inner(controls_area));
        frame.render_widget(controls_block, controls_area);

        if self.play_disabled {
            render_size_warning(frame);
        }
    }

    pub fn handle_keypress(&mut self, _state: &mut State, key: &KeyEvent) {
        let online = online();
        let online = online.as_ref();
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = self.selected.previous(),
            KeyCode::Down | KeyCode::Char('s') => self.selected = self.selected.next(),
            KeyCode::Enter if self.selected != MenuChoice::Play || !self.play_disabled => {
                self.activated = Some(self.selected)
            }
            KeyCode::Char('p') if !self.play_disabled => self.activated = Some(MenuChoice::Play),
            KeyCode::Char('o') => self.activated = Some(MenuChoice::Options),
            KeyCode::Char('q') => self.activated = Some(MenuChoice::Quit),
            KeyCode::Left | KeyCode::Char('a') if online.is_some() => {
                self.leaderboard_offset = self.leaderboard_offset.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('d') if let Some(online) = online => {
                self.leaderboard_offset =
                    (self.leaderboard_offset + 1).min(online.entries().len().saturating_sub(20));
            }
            _ => {}
        }
    }

    pub fn update(&self, state: &mut State) -> Option<AppScreen> {
        match self.activated {
            Some(MenuChoice::Play) if !self.play_disabled => {
                *state = State::default();
                Some(Game(GameScreen::default()))
            }
            Some(MenuChoice::Play) => None,
            Some(MenuChoice::Quit) => Some(Quit),
            Some(MenuChoice::Options) => Some(Options(OptionsScreen::default())),
            None => None,
        }
    }

    fn clamp_leaderboard_offset(&mut self, entry_count: usize) {
        self.leaderboard_offset = self.leaderboard_offset.min(entry_count.saturating_sub(10));
    }

    fn draw_leaderboard(&self, frame: &mut Frame, horiz: Rect, online: &Online) {
        let [area] = Layout::vertical([Constraint::Length(22)]).areas(horiz);
        let entries = online.entries();
        let end = (self.leaderboard_offset + 20).min(entries.len());
        let title = if entries.is_empty() {
            "Leaderboard".into()
        } else {
            format!(
                "Leaderboard {}-{}/{LEADERBOARD_LIMIT}",
                self.leaderboard_offset + 1,
                end
            )
        };
        let lines = if let Some(error) = online.load_error() {
            vec![Line::styled(
                format!("Unavailable: {error}"),
                Style::default().fg(Color::LightRed),
            )]
        } else if entries.is_empty() {
            vec![Line::from("No scores yet")]
        } else {
            (self.leaderboard_offset..end)
                .map(|rank| {
                    let entry = &entries[rank];
                    let style =
                        (|s: Style| {
                            if rank == 0 {
                                s.fg(Color::LightYellow) // gold
                            } else if rank == 1 {
                                s.fg(Color::Rgb(192, 192, 192)) // silver
                            } else if rank == 2 {
                                s.fg(Color::Rgb(205, 127, 50)) // bronze
                            } else if entry.is_current {
                                s
                            } else {
                                s.fg(Color::Gray).remove_modifier(Modifier::BOLD)
                            }
                        })(Style::default().add_modifier(Modifier::BOLD));
                    Line::styled(
                        format!(
                            "{:>2} {:<12} {:>10}",
                            entry.rank, entry.username, entry.score
                        ),
                        style,
                    )
                })
                .collect()
        };
        let block = Block::bordered()
            .title(title)
            .title_bottom("A/D scroll")
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(Paragraph::new(lines), block.inner(area));
        frame.render_widget(block, area);
    }
}
