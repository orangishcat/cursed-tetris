mod input;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

#[derive(Default)]
pub struct App {
    count: i32,
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(frame.area());

        let title = Paragraph::new("Ratatui Counter")
            .alignment(Alignment::Center)
            .block(Block::new().borders(Borders::ALL));

        let counter = Paragraph::new(self.count.to_string())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::bordered().title(" Count "));

        let help = Paragraph::new(Line::from(
            "Left/Right: change count | r: reset | q: quit",
        ))
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::ALL));

        frame.render_widget(title, header);
        frame.render_widget(counter, body);
        frame.render_widget(help, footer);
    }

    pub fn handle_event(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            // Some platforms emit Press, Repeat, and Release events.
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key.code {
                KeyCode::Left => self.count -= 1,
                KeyCode::Right => self.count += 1,
                KeyCode::Char('r') => self.count = 0,
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                _ => {}
            }
        }
        Ok(())
    }
}
