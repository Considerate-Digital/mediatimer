use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect, Flex},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};
use std::error::Error;
use crate::ProcType;

const ITEM_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;

use crate::Autoloop;

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
    pub fn run (mut self, mut terminal: &mut DefaultTerminal) -> Result<(), Box< dyn Error>> {
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

    fn render_list(area: Rect, buf: &mut Buffer) {
        let title = Line::raw("Welcome to medialoop").centered();
        let para = Paragraph::new("Medialoop was created by Considerate Digital to help automate and control things in exhibition spaces.");
        let length = title.width() * 4;
        let block = Block::new()
            .title(title.clone())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(ALT_ROW_BG_COLOR)
            .render(
                center(area, 
                    Constraint::Length(length as u16), 
                    Constraint::Length(4)
                ),
            buf
            );
    }

}
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical])
        .flex(Flex::Center)
        .areas(area);
    area
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
        LandingWidget::render_list(center_area, buf);
    }

}

