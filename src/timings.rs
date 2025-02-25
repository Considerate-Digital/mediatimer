use derive_setters::Setters;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        Color, Stylize, Style
    },
    symbols,
    text::{Line, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Clear,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};

use std::error::Error;
use crate::Timings;
use crate::Weekday;

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    ALT_ROW_BG_COLOR,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
};

#[derive(Debug, Default, Setters)]
struct Popup<'a> {
    #[setters(into)]
    title: Line<'a>,
    #[setters(into)]
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);

        Paragraph::new(self.content)
            .wrap(Wrap { trim:true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}

pub struct TimingsWidget {
    should_exit: bool,
    list_element_entries: TimingsList,
    schedule: Vec<Weekday>
}

struct TimingsList {
    list: Vec<TimingsEntry>,
    state: ListState
}

impl FromIterator<(Weekday, &'static str)> for TimingsList {
    fn from_iter<I: IntoIterator<Item = (Weekday, &'static str)>>(iter: I) -> Self {
        let list = iter
            .into_iter()
            .map(|(list_element, info)| TimingsEntry::new(list_element, info))
            .collect();
        let state = ListState::default();
        Self { list, state }
    }
}
struct TimingsEntry {
    list_element: String,
    timings: Vec<(String, String)>,
    info: String,
}

impl From<&TimingsEntry> for ListItem<'_> {
    fn from(value: &TimingsEntry) -> Self {
        let line = Line::styled(format!("{}", value.list_element.as_str()), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

impl TimingsEntry {
    fn new(weekday: Weekday, info: &str) -> Self {
        Self {
            list_element: weekday.to_string(),
            timings: weekday.timings(),
            info: info.to_string()
        }
    }
}

impl Default for TimingsWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
            list_element_entries: TimingsList::from_iter([
                (Weekday::Monday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Tuesday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Wednesday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Thursday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Friday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Saturday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
                (Weekday::Sunday(vec![(String::from("09:00"), String::from("17:00"))]), "Enter the start and end timings for this day."),
            ]),
            schedule: Vec::with_capacity(7)
        }
    }
}

impl TimingsWidget {
    pub fn run (mut self, mut terminal: &mut DefaultTerminal) -> Result<Timings, Box< dyn Error>> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
                //self.text_area.input(key);
            };
        }
        Ok(self.schedule)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                // add code to select the list item
                // render popup now using current selection
            }
            _ => {}
        }
    }
    
    fn select_none(&mut self) {
        self.list_element_entries.state.select(None);
    }
    fn select_next(&mut self) {
        self.list_element_entries.state.select_next();
    }
    fn select_previous(&mut self) {
        self.list_element_entries.state.select_previous();
    }
    fn select_first(&mut self) {
        self.list_element_entries.state.select_first();
    }
    fn select_last(&mut self) {
        self.list_element_entries.state.select_last();
    }


    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("What do you want to run?")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Select your file type").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the elements in the 'items' and stylise them
        let items: Vec<ListItem> = self 
            .list_element_entries
            .list
            .iter()
            .enumerate()
            .map(|(i, list_item)| {
                let color = alternate_colors(i);
                ListItem::from(list_item).bg(color)
            })
            .collect();

        // create a list from all the items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.list_element_entries.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // get the info
        let info = if let Some(i) = self.list_element_entries.state.selected() {
            self.list_element_entries.list[i].info.clone()
        } else {
            "Nothing selected...".to_string()
        };

        // show the list item's info under the list
        let block = Block::new()
            .title(Line::raw("TYPE INFO").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl Widget for &mut TimingsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] = Layout::vertical([
            Constraint::Fill(3),
            Constraint::Fill(1)
        ])
        .areas(main_area);

        TimingsWidget::render_header(header_area, buf);
        TimingsWidget::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);

        if 1 > 0 {
            let popup_area = Rect {
                x: area.width / 4,
                y: area.height / 3,
                width: area.width / 2,
                height: area.height / 3,
            };

            let popup = Popup::default()
                .content("hello")
                .style(SELECTED_STYLE)
                .title("Select a timing")
                .border_style(ITEM_HEADER_STYLE)
                .render(popup_area, buf);


        }
    }
}

