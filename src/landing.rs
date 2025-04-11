use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect, Flex },
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

use crate::areas::{
    popup_area
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
        Paragraph::new("Media Timer Setup")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Press Enter or ESC to continue.")
            .centered()
            .render(area, buf);
    }

    fn render_logo(area: Rect, buf: &mut Buffer) {
        /*
        let logo = indoc::indoc! {"
        ⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡏⠉⠉⠉⠉⠉⢹⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⠸⠿⠇⠀⠀⠀⠀⠀⠸⠿⠇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢰⣶⡆⠀⠀⠀⠀⠀⢰⣶⡆⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣷⣶⣶⣶⣶⣶⣾⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⣤⣤⠀⠀⠀⠀⠘⠛⠛⠛⠛⠛⠛⠛⠛⠛⠃⠀⠀⠀⠀⣤⣤⠀⠀⣿
        ⣿⠀⠀⠛⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⠛⠀⠀⣿
        ⣿⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣿
        "};
        */
        
        let logo = indoc::indoc! {"
        ⣿⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⣿
        ⣿⠀⢾⡷⠀⠀⠀⣀⣀⣀⣀⣀⣀⣀⠀⠀⠀⢾⡷⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⣿⡿⠿⠿⠿⢿⣿⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⣿⡇⠀⠀⠀⢸⣿⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⠛⠃⠀⠀⠀⠘⠛⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⣤⡄⠀⠀⠀⢠⣤⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⣿⡇⠀⠀⠀⢸⣿⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⠀⠀⠀⠀⠀⣿⣇⣀⣀⣀⣸⣿⠀⠀⠀⠀⠀⠀⣿
        ⣿⠀⣠⣄⠀⠀⠀⠿⠿⠿⠿⠿⠿⠿⠀⠀⠀⣠⣄⠀⣿
        ⣿⠀⠙⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠋⠀⣿
        ⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛
        "};
    /*
        let logo = indoc::indoc! {"


            ⠀⠀⠀⠀⠀⠀⣀⣀⣤⡶⠶⠶⠶⠶⠶⢦⣤⣀⣀⠀⠀⢰⡆
            ⠀⠀⠀⢀⣴⡿⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⢿⣦⣸⡇
            ⠀⠀⣰⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣶⣶⣶⣿⣿⡇
            ⠀⣼⡟
            ⢰⡟
            ⢸⡇
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠃
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠏
            ⠀⠀⢸⣿⣿⠿⠿⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⠏
            ⠀⠀⢸⡏⠻⣷⣄⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣾⠟⠁
            ⠀⠀⢸⡇⠀⠀⠙⠻⠿⣶⣶⣶⣤⣶⣶⣾⠿⠛⠉

        "};
        let _big_logo = indoc::indoc! {"

            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣤⣤⣶⣶⠶⣶⣶⣤⣤⣄⣀⠀⠀⠀⠀⠀⠀⣤
            ⠀⠀⠀⠀⠀⠀⠀⢀⣤⡶⠟⠛⠉⠁⠀⠀⠀⠀⠀⠀⠀⠈⠉⠛⠻⣶⣄⡀⠀⠀⣿
            ⠀⠀⠀⠀⠀⣠⣴⠟⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠻⣦⣄⣿
            ⠀⠀⠀⢀⣼⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣤⣤⣤⣬⣿⣿
            ⠀⠀⢠⡿⠋
            ⠀⢠⡿⠁
            ⠀⣾⠃
            ⢰⡿
            ⠸⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⡇
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣿⠁
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⡏
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⡟
            ⠀⠀⠀⠀⣤⣤⣤⣤⣤⣤⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⠟
            ⠀⠀⠀⠀⣿⢿⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⡿⠁
            ⠀⠀⠀⠀⣿⠀⠙⠻⣶⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⡾⠟⠉
            ⠀⠀⠀⠀⣿⠀⠀⠀⠀⠉⠛⠿⣶⣦⣤⣤⣤⣀⣤⣤⣤⣴⡶⠿⠛⠉
            ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠉⠉⠉⠁

        "};
    */
        let logo_text = Text::styled(logo, Color::Rgb(255, 255, 255));
        let area = centered_rect(area, logo_text.width() as u16, logo_text.height() as u16);

        logo_text.render(area, buf);

    }
    fn render_text(area: Rect, buf: &mut Buffer) {

        let _para = Paragraph::new(
            vec![
                Line::from(
                        "Media Timer was created by Considerate Digital to automate"
                ),
                Line::from(
                        "and control players in exhibition spaces."
                ),

                Line::from(""),
                Line::from(
                        "Press ENTER to start."
                ),
            ])
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(
                area,
                buf
            );
    }
    fn render_center(area: Rect, buf: &mut Buffer) {
        let title = Line::raw("Media Timer").centered();
        let _length = title.width() * 4;

        let block = Block::new()
            .title(title.clone())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .padding(Padding::uniform(4))
            .bg(ALT_ROW_BG_COLOR);

        // render block
        block.render(area, buf);
    
        // split the center into two areas
        let [logo_area, text_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1)
        ])
        .areas(area);


        // render logo inside block
        LandingWidget::render_logo(logo_area, buf);
        // render text inside block
        LandingWidget::render_text(text_area, buf);
    }

}
/// a centered rect of the given size
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
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

                LandingWidget::render_header(header_area, buf);
        LandingWidget::render_footer(footer_area, buf);
        LandingWidget::render_center(main_area, buf);
        /*
        LandingWidget::render_logo(logo_area, buf);
        LandingWidget::render_text(text_area, buf);
        */
    }
}

