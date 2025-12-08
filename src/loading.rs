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
use std::{
    process::{
        Command
    },
    thread,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering

        },
    }
};

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    TEXT_FG_COLOR,
};

pub struct LoadingWidget {
    should_exit: Arc<AtomicBool>,
}

impl Default for LoadingWidget {
    fn default() -> Self {
        Self {
            should_exit: Arc::new(AtomicBool::new(false))
        }
    }
}

impl LoadingWidget {
    pub fn new() -> Self {
        Self {
            should_exit: Arc::new(AtomicBool::new(false))
        }
    }
    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<(), Box< dyn Error>> {
        // set the command going
        let should_exit_clone = Arc::clone(&self.should_exit);
        self.run_command(should_exit_clone);
        while !self.should_exit.load(Ordering::Relaxed) {
            terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }


    fn run_command(&self, should_exit: Arc<AtomicBool>) {
        thread::spawn(move|| {
            let _enable_mediatimer_init = Command::new("systemctl")
                .arg("--user")
                .arg("start")
                .arg("mediatimer_init.service")
                .output()
                .expect("Media Timer not restarted");

            should_exit.store(true, Ordering::Relaxed);
        });
    }
        

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {},//self.should_exit = true,
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                // add code to select the list item
                //self.should_exit = true;
            }
            _ => {}
        }
    }
    


    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Loading")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("")
            .centered()
            .render(area, buf);
    }

    fn render_text(area: Rect, buf: &mut Buffer) {
        let title = Line::raw("Loading").centered();
        let _length = title.width() * 4;
        let block = Block::new()
            .title(title.clone())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .padding(Padding::uniform(4))
            .bg(NORMAL_ROW_BG);

        Paragraph::new(
            vec![
                Line::from("Please Wait"
                ),
                Line::from("Loading the configuration..."
                ),
                Line::from(""),
            ])
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .fg(TEXT_FG_COLOR)
            .render(
                area,
                buf
            );
    }

}

impl Widget for &mut LoadingWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let [header_area, _, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        LoadingWidget::render_header(header_area, buf);
        LoadingWidget::render_footer(footer_area, buf);
        LoadingWidget::render_text(area, buf);
    }
}

