use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect },
    style::{
        Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, Padding, Paragraph,
        Widget, Wrap
    },
    DefaultTerminal,
};
use ratatui::prelude::*;
use std::error::Error;

use crate::styles::{
    ITEM_HEADER_STYLE,
    ALT_ROW_BG_COLOR,
};


pub struct LandingWidget {
    should_exit: bool,
}

impl Default for LandingWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
        }
    }
}

impl LandingWidget {
    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<(), Box< dyn Error>> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                // add code to select the list item
                self.should_exit = true;
            }
            _ => {}
        }
    }
    


    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Medialoop Setup")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Press Enter or ESC to continue.")
            .centered()
            .render(area, buf);
    }

    fn render_text(area: Rect, buf: &mut Buffer) {
        let title = Line::raw("Welcome to medialoop!").centered();
        let _length = title.width() * 4;
        let block = Block::new()
            .title(title.clone())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .padding(Padding::uniform(4))
            .bg(ALT_ROW_BG_COLOR);

        let _para = Paragraph::new(
            vec![
                Line::from(
                        "Medialoop was created by Considerate Digital to help automate and control things in exhibition spaces."
                ),
                Line::from(""),
                Line::from(
                        "Let's get going!"
                ),
            ])
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(
                area,
                buf
            );
    }

}

impl Widget for &mut LandingWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [center_area] = Layout::vertical([
            Constraint::Fill(1)
        ])
        .areas(main_area);
        LandingWidget::render_header(header_area, buf);
        LandingWidget::render_footer(footer_area, buf);
        LandingWidget::render_text(center_area, buf);
    }
}

