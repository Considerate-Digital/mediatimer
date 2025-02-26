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
enum CurrentScreen {
    Weekdays,
    Day,
    TimingOptions,
    Add,
    Edit,
    Delete,
    Exit
}

#[derive(Debug, Clone)]
struct Timing {
    timing: (String, String)
}

impl Timing {
    fn default() -> Timing {
        Timing {
            timing: (String::from("09:00"), String::from("17:00"))
        }
    }
    fn format(self) -> String {
        let mut joined = String::from(&self.timing.0);
        joined.push_str(&self.timing.1);
        joined
    }
}

impl From<&Timing> for ListItem<'_> {
    fn from(value: &Timing) -> Self {
        let line = Line::styled(format!("{}", value.clone().format()), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}
#[derive(Debug, Clone)]
pub struct TimingCollection {
    timing_collection: Vec<Timing>,
    state: ListState
}

impl TimingCollection {
    fn default() -> TimingCollection {
        let mut state = ListState::default();
        state.select_first();

        TimingCollection {
            timing_collection: vec![Timing::default()],
            state
        }
    }
}

enum TimingOp {
    Add,
    Del,
    Edit
}

impl TimingOp {
    fn as_str(&self) -> &'static str {
        match self {
            TimingOp::Add => "Add",
            TimingOp::Del => "Delete",
            TimingOp::Edit => "Edit"
        }
    }
    fn as_vec_of_str(&self) -> Vec<TimingOpItem> {
        vec![
            TimingOpItem::from("Add"), 
            TimingOpItem::from("Delete"), 
            TimingOpItem::from("Edit")
        ]
    }
}

struct TimingOpList {
    timing_ops: Vec<TimingOpItem>,
    state: ListState
}

impl TimingOpList {
    fn default() -> TimingOpList {
        let mut state = ListState::default();
        state.select_first();
        TimingOpList {
            timing_ops: TimingOp::Add.as_vec_of_str(),
            state
        }
    }
}

struct TimingOpItem {
    op_item: String
}

impl TimingOpItem {
    fn from(new: &str) -> TimingOpItem {
        TimingOpItem {
            op_item: String::from(new)
        }
    }
}

impl From<&TimingOpItem> for ListItem<'_> {
    fn from(value: &TimingOpItem) -> Self {
        let line = Line::styled(format!("{}", value.op_item), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

pub struct TimingsWidget {
    should_exit: bool,
    current_screen: CurrentScreen,
    // selected weekdays and timings are indexes
    weekday_selected: usize,
    timing_selected: usize,
    operation_selected: TimingOp,
    timing_op_list: TimingOpList,
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
        let mut state = ListState::default();
        state.select_first();
        Self { list, state }
    }
}

struct TimingsEntry {
    list_element: String,
    timings: TimingCollection,
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
            current_screen: CurrentScreen::Weekdays,
            weekday_selected: 0,
            timing_selected: 0,
            operation_selected: TimingOp::Add,
            timing_op_list: TimingOpList::default(),
            list_element_entries: TimingsList::from_iter([
                (Weekday::Monday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Tuesday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Wednesday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Thursday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Friday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Saturday(TimingCollection::default()), "Enter the start and end timings for this day."),
                (Weekday::Sunday(TimingCollection::default()), "Enter the start and end timings for this day."),
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
        match self.current_screen {
            CurrentScreen::Weekdays => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                    //KeyCode::Char('h') | KeyCode::Left => self.select_none(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        // check the current selection
                        let info = if let Some(i) = self.list_element_entries.state.selected() {
                            self.weekday_selected = i;
                        };

                        // change state to DAY
                        self.current_screen = CurrentScreen::Day;

                    }
                    _ => {}
                }
            },
            CurrentScreen::Day => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => self.reverse_state(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        let _change_selected_timing = if let Some(i) = self.list_element_entries.state.selected() {
                            self.timing_selected = i;
                        };

                        // change state to Timing Options
                        self.current_screen = CurrentScreen::TimingOptions;
                    }
                    _ => {}
                }
            },
            CurrentScreen::TimingOptions => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => self.reverse_state(),
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
            },
            CurrentScreen::Add => {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => self.reverse_state(),
                    KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                    }
                    _ => {}
                }
            },
            CurrentScreen::Edit => {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => self.reverse_state(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                    }
                    _ => {}
                }
            },

            CurrentScreen::Delete => {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => self.reverse_state(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                    }
                    _ => {}
                }
            },
            CurrentScreen::Exit => {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => self.reverse_state(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                    }
                    _ => {}
                }

            }
        }
    }
    
    /*
    match self.current_screen {
        CurrentScreen::Weekdays => {},
        CurrentScreen::Day => {},
        CurrentScreen::TimingOptions => {},
        CurrentScreen::Add => {},
        CurrentScreen::Edit => {},
        CurrentScreen::Delete => {},
        CurrentScreen::Exit => {}
    }
    */

    fn reverse_state(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.current_screen = CurrentScreen::Weekdays,
            CurrentScreen::Day => self.current_screen = CurrentScreen::Weekdays,
            CurrentScreen::TimingOptions => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Add => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Edit => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Delete => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Exit => self.current_screen = CurrentScreen::Day
        }

    }

    fn select_next(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_next(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_next(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_next(),
            CurrentScreen::Add => {},
            CurrentScreen::Edit => {},
            CurrentScreen::Delete => {},
            CurrentScreen::Exit => {}
        }
    }
    fn select_previous(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_previous(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_previous(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_previous(),
            CurrentScreen::Add => {},
            CurrentScreen::Edit => {},
            CurrentScreen::Delete => {},
            CurrentScreen::Exit => {}
        }

    }
    fn select_first(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_first(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_first(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_first(),
            CurrentScreen::Add => {},
            CurrentScreen::Edit => {},
            CurrentScreen::Delete => {},
            CurrentScreen::Exit => {}
        }

    }
    fn select_last(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_last(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_last(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_last(),
            CurrentScreen::Add => {},
            CurrentScreen::Edit => {},
            CurrentScreen::Delete => {},
            CurrentScreen::Exit => {}
        }


    }


    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Make the schedule")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_weekdays_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Select a day").centered())
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
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.list_element_entries.state);
    }

    fn render_day_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Edit the schedule").centered())
            .borders(Borders::TOP | Borders::LEFT)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .list_element_entries
            .list[self.weekday_selected]
            .timings.timing_collection
            .iter()
            .enumerate()
            .map(|(i, timings)| {
                let color = alternate_colors(i);
                let mut timings_joined = String::from(&timings.timing.0);
                timings_joined.push_str("-");
                timings_joined.push_str(&timings.timing.1);
                
                ListItem::from(timings_joined).bg(color)
            })
            .collect();

        // create a list from all the items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.list_element_entries.list[self.weekday_selected].timings.state);
    }

    fn render_op_list(&mut self, area: Rect, buf: &mut Buffer) {
        /*
        let popup = Popup::default()
                .content("hello")
                .style(SELECTED_STYLE)
                .title("Select a timing")
                .border_style(ITEM_HEADER_STYLE)
                .render(popup_area, buf);
        */

        let block = Block::new()
            .title(Line::raw("Select an operation").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .timing_op_list
            .timing_ops
            .iter()
            .enumerate()
            .map(|(i, ops)| {
                let color = alternate_colors(i);
                ListItem::from(ops).bg(color)
            })
            .collect();

        // create a list from all the items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.timing_op_list.state);

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

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        match self.current_screen {
            CurrentScreen::Weekdays => {
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
                self.render_weekdays_list(list_area, buf);
                self.render_selected_item(item_area, buf);

            },
            CurrentScreen::Day => {
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

                let [weekdays_area, day_area] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Fill(1)
                ])
                .areas(list_area);

                TimingsWidget::render_header(header_area, buf);
                TimingsWidget::render_footer(footer_area, buf);
                self.render_weekdays_list(weekdays_area, buf);
                self.render_day_list(day_area, buf);
                self.render_selected_item(item_area, buf);

            },
            CurrentScreen::TimingOptions => {
                self.render_op_list(popup_area, buf);
           },
            CurrentScreen::Add => {

                Clear.render(area, buf);
            },
            CurrentScreen::Edit => {},
            CurrentScreen::Delete => {},
            CurrentScreen::Exit => {},
        }
    }
}

