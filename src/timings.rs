use derive_setters::Setters;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect, Position},
    style::{
        Color, Stylize, Style, Modifier
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
use std::process;
use crate::Timings;
use crate::Weekday as CommonWeekday;
use crate::Schedule as CommonSchedule;
use crate::Timings as CommonTimings;
use regex::Regex;
use strum::Display;

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    ALT_ROW_BG_COLOR,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
    FOOTER_STYLE
};

// This is declared twice due to the TUI list structure requirements and must be converted 
// between main and this module
#[derive(Display, Debug)]
pub enum Weekday {
    Monday(TimingCollection),
    Tuesday(TimingCollection),
    Wednesday(TimingCollection),
    Thursday(TimingCollection),
    Friday(TimingCollection),
    Saturday(TimingCollection),
    Sunday(TimingCollection),
}

impl Weekday {
    fn as_str(&self) -> &'static str {
        match self {
            Weekday::Monday(_) => "Monday",
            Weekday::Tuesday(_) => "Tuesday",
            Weekday::Wednesday(_) => "Wednesday",
            Weekday::Thursday(_) => "Thursday",
            Weekday::Friday(_) => "Friday",
            Weekday::Saturday(_) => "Saturday",
            Weekday::Sunday(_) => "Sunday"
        }
    }
    fn to_string(&self) -> String {
        match self {
            Weekday::Monday(_) => String::from("Monday"),
            Weekday::Tuesday(_) => String::from("Tuesday"),
            Weekday::Wednesday(_) => String::from("Wednesday"),
            Weekday::Thursday(_) => String::from("Thursday"),
            Weekday::Friday(_) => String::from("Friday"),
            Weekday::Saturday(_) => String::from("Saturday"),
            Weekday::Sunday(_) => String::from("Sunday")
        }
    }

    fn timings(&self) -> TimingCollection {
        match self {
            Weekday::Monday(schedule) => schedule.clone(),
            Weekday::Tuesday(schedule) => schedule.clone(),
            Weekday::Wednesday(schedule) => schedule.clone(),
            Weekday::Thursday(schedule) => schedule.clone(),
            Weekday::Friday(schedule) => schedule.clone(),
            Weekday::Saturday(schedule) => schedule.clone(),
            Weekday::Sunday(schedule) => schedule.clone()
        }
    }
}

fn common_to_local_weekday(wd: CommonWeekday) -> Weekday {
    match wd {
        CommonWeekday::Monday(schedule) => Weekday::Monday(TimingCollection::from_common_schedule(schedule)), 
        CommonWeekday::Tuesday(schedule) => Weekday::Tuesday(TimingCollection::from_common_schedule(schedule)),
        CommonWeekday::Wednesday(schedule) => Weekday::Wednesday(TimingCollection::from_common_schedule(schedule)),
        CommonWeekday::Thursday(schedule) => Weekday::Thursday(TimingCollection::from_common_schedule(schedule)),
        CommonWeekday::Friday(schedule) => Weekday::Friday(TimingCollection::from_common_schedule(schedule)),
        CommonWeekday::Saturday(schedule) => Weekday::Saturday(TimingCollection::from_common_schedule(schedule)), 
        CommonWeekday::Sunday(schedule) => Weekday::Sunday(TimingCollection::from_common_schedule(schedule)),
    }

}

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
#[derive(Debug, Clone, PartialEq)]
enum CurrentScreen {
    Weekdays,
    Day,
    TimingOptions,
    Add,
    Edit,
    Delete,
    Error,
    Exit
}

#[derive(Debug, Clone)]
struct Timing {
    timing: (String, String)
}

impl Timing {
    fn default() -> Timing {
        Timing {
            timing: (String::from("09:00:00"), String::from("17:00:00"))
        }
    }
    fn new(start: &str, end: &str) -> Timing {
        Timing {
            timing: (String::from(start), String::from(end))
        }
    }
    fn format(self) -> String {
        let mut joined = String::from(&self.timing.0);
        joined.push_str("-");
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
    fn from_common_schedule(schedule: CommonSchedule) -> TimingCollection {
        let mut state = ListState::default();
        state.select_first();
        let mut timing_collection = Vec::with_capacity(1);
        for t in schedule.iter() {
            let timing = Timing::new(&t.0, &t.1);
            timing_collection.push(timing);
        }
        TimingCollection {
            timing_collection,
            state
        }
    }
}

#[derive(Clone)]
enum TimingOp {
    Add,
    Del,
    Edit,
    Exit
}

impl TimingOp {
    fn as_str(&self) -> &'static str {
        match self {
            TimingOp::Add => "Add",
            TimingOp::Del => "Delete",
            TimingOp::Edit => "Edit",
            TimingOp::Exit => "Exit"

        }
    }
    fn as_vec_of_str(&self) -> Vec<TimingOpItem> {
        vec![
            TimingOpItem::from("Add"), 
            TimingOpItem::from("Delete"), 
            TimingOpItem::from("Edit"),
            TimingOpItem::from("Exit")
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
struct DelOpItem {
    item: String
}
impl DelOpItem {
    fn new(item: &str) -> DelOpItem {
        DelOpItem {
            item: String::from(item)
        }
    }
}
struct DelOpList {
    del_list: Vec<DelOpItem>,
    state: ListState
}
impl DelOpList {
    fn default() -> DelOpList {
        let mut state = ListState::default();
        state.select_first();
        DelOpList {
            del_list: vec![DelOpItem::new("Yes"), DelOpItem::new("No")],
            state
        }
    }
}

impl From<&DelOpItem> for ListItem<'_> {
    fn from(value: &DelOpItem) -> Self {
        let line = Line::styled(format!("{}", value.item), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}
struct ExitItem {
    item: String
}
impl ExitItem {
    fn new(item: &str) -> ExitItem {
        ExitItem {
            item: String::from(item)
        }
    }
}
struct ExitList {
    exit_list_items: Vec<ExitItem>,
    state: ListState
}
impl ExitList {
    fn default() -> ExitList {
        let mut state = ListState::default();
        state.select_first();
        ExitList {
            exit_list_items: vec![ExitItem::new("Yes"), ExitItem::new("No")],
            state
        }
    }
}

impl From<&ExitItem> for ListItem<'_> {
    fn from(value: &ExitItem) -> Self {
        let line = Line::styled(format!("{}", value.item), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

enum ErrorType {
    Format,
    Clash
}


pub struct TimingsWidget {
    should_exit: bool,
    current_screen: CurrentScreen,
    previous_screen: CurrentScreen,
    // selected weekdays and timings are indexes
    weekday_selected: usize,
    timing_selected: usize,
    operation_selected: TimingOp,
    timing_op_list: TimingOpList,
    input: String,
    character_index: usize,
    input_area: Rect,
    del_op_list: DelOpList,
    exit_list: ExitList,
    error_type: ErrorType,
    list_element_entries: TimingsList,
    schedule: CommonTimings
}

struct TimingsList {
    list: Vec<TimingsEntry>,
    state: ListState
}
impl Default for TimingsList {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select_first();
        Self {
            list: Vec::with_capacity(1),
            state
        }
    }
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
        let info_text = "Enter the start and end timings for this day. Use ENTER or the right keyboard arrow → to advance through the menu. Add, edit or delete schedule timings. Use ESC or the left keyboard arrow ← to retreat through the menus and exit. Schedule timings must use the 24 hour clock and must follow the format 00:00:00-00:00:00";
        Self {
            should_exit: false,
            current_screen: CurrentScreen::Weekdays,
            previous_screen: CurrentScreen::Weekdays,
            weekday_selected: 0,
            timing_selected: 0,
            operation_selected: TimingOp::Add,
            timing_op_list: TimingOpList::default(),
            input: String::new(),
            character_index: 0,
            input_area: Rect::new(0,0,0,0),
            del_op_list: DelOpList::default(),
            exit_list: ExitList::default(),
            error_type: ErrorType::Format,
            list_element_entries: TimingsList::from_iter([
                (Weekday::Monday(TimingCollection::default()), info_text),
                (Weekday::Tuesday(TimingCollection::default()), info_text),
                (Weekday::Wednesday(TimingCollection::default()), info_text),
                (Weekday::Thursday(TimingCollection::default()), info_text),
                (Weekday::Friday(TimingCollection::default()), info_text),
                (Weekday::Saturday(TimingCollection::default()), info_text),
                (Weekday::Sunday(TimingCollection::default()), info_text),
            ]),
            schedule: Vec::with_capacity(7)
        }
    }
}

fn parse_common_timings(c_timings: CommonTimings) -> TimingsList {
    let info_text = "Enter the start and end timings for this day. Use ENTER or the right keyboard arrow to advance through the menu. Add, edit or delete schedule timings. Use ESC or the left keyboard arrow to retreat through the menus. Schedule timings must use the 24 hour clock and must follow the format 00:00:00-00:00:00";

    let days_with_timings_collection: Vec<_> = c_timings.iter()
        .map(|ct| common_to_local_weekday(ct.clone()))
        .map(|schedule| schedule.timings())
        .filter(|t| t.timing_collection.len() > 0)
        .collect();
    let days_without_timings_count = 7 - days_with_timings_collection.len(); 


    //type Timings = Vec<Weekday>;
    if c_timings.len() == 7 && days_without_timings_count != 7 {
        let mut t_list = TimingsList::default();
        for day in c_timings.iter() {
            let day = common_to_local_weekday(day.clone());
            let t_entry = TimingsEntry::new(day, info_text);
            t_list.list.push(t_entry); 
        }
        t_list

    } else {
        TimingsList::from_iter([
            (Weekday::Monday(TimingCollection::default()), info_text),
            (Weekday::Tuesday(TimingCollection::default()), info_text),
            (Weekday::Wednesday(TimingCollection::default()), info_text),
            (Weekday::Thursday(TimingCollection::default()), info_text),
            (Weekday::Friday(TimingCollection::default()), info_text),
            (Weekday::Saturday(TimingCollection::default()), info_text),
            (Weekday::Sunday(TimingCollection::default()), info_text),
        ])
    }
}

impl TimingsWidget {
    pub fn new (preset_timings: CommonTimings) -> Self {

        // convert the common-timings to timings
        let parsed_timings: TimingsList = parse_common_timings(preset_timings);


        Self {
            should_exit: false,
            current_screen: CurrentScreen::Weekdays,
            previous_screen: CurrentScreen::Weekdays,
            weekday_selected: 0,
            timing_selected: 0,
            operation_selected: TimingOp::Add,
            timing_op_list: TimingOpList::default(),
            input: String::new(),
            character_index: 0,
            input_area: Rect::new(0,0,0,0),
            del_op_list: DelOpList::default(),
            exit_list: ExitList::default(),
            error_type: ErrorType::Format,
            list_element_entries: parsed_timings,
            schedule: Vec::with_capacity(7)
        }
    }
    pub fn run (mut self, mut terminal: &mut DefaultTerminal) -> Result<Timings, Box< dyn Error>> {

        while !self.should_exit {
            terminal.draw(|f| {
                f.render_widget(&mut self, f.area());
                match self.current_screen {
                    CurrentScreen::Add | CurrentScreen::Edit => {
                        f.set_cursor_position(Position::new(
                                self.input_area.x + self.character_index as u16 + 1,
                                // move one line down, from the border to the input lin
                                self.input_area.y + 1,
                        ))
                    },
                    _ => {}
                }
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
                //self.text_area.input(key);
            };
        }
        Ok(self.schedule)
    }

    fn compile_schedule(&mut self) {
        for t_list in &self.list_element_entries.list {
            let mut cs: CommonSchedule = Vec::with_capacity(1);
            for t in t_list.timings.timing_collection.iter() {
                let (start, end) = &t.timing;
                let schedule_item = (
                    String::from(&start.to_string()),
                    String::from(&end.to_string())
                );
                cs.push(schedule_item);
            }
            
            // here we are converting to the data structure expected in main
            let wd = match t_list.list_element.as_str() {
                "Monday" => CommonWeekday::Monday(cs.clone()),
                "Tuesday" => CommonWeekday::Tuesday(cs.clone()),
                "Wednesday" => CommonWeekday::Wednesday(cs.clone()),
                "Thursday" => CommonWeekday::Thursday(cs.clone()),
                "Friday" => CommonWeekday::Friday(cs.clone()),
                "Saturday" => CommonWeekday::Saturday(cs.clone()),
                "Sunday" => CommonWeekday::Sunday(cs.clone()),
                _ => CommonWeekday::Monday(cs.clone()),

            };
            self.schedule.push(wd);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match self.current_screen {
            CurrentScreen::Weekdays => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Left | KeyCode::Backspace | KeyCode::Esc => self.current_screen = CurrentScreen::Exit,
                    //KeyCode::Char('h') | KeyCode::Left => self.select_none(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        // check the current selection
                        if let Some(i) = self.list_element_entries.state.selected() {
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
                    KeyCode::Char('q') | KeyCode::Esc => self.current_screen = CurrentScreen::Exit,
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => self.reverse_state(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        if let Some(i) = self.list_element_entries.list[self.weekday_selected].timings.state.selected() {
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
                    KeyCode::Char('q') | KeyCode::Esc => self.reverse_state(),
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => self.reverse_state(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),

                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        if let Some(i) = self.timing_op_list.state.selected() {
                            
                            let op = match i {
                                0 => TimingOp::Add,
                                1 => TimingOp::Del,
                                2 => TimingOp::Edit,
                                3 => TimingOp::Exit,
                                _ => TimingOp::Add

                            };

                            self.operation_selected = op.clone();

                            match op {
                                TimingOp::Add => self.current_screen = CurrentScreen::Add,
                                TimingOp::Del => self.current_screen = CurrentScreen::Delete,
                                TimingOp::Edit => {
                                    if self.list_element_entries.list[self.weekday_selected].timings.timing_collection.len() > 0 {
                                        self.input.clear();
                                        self.input.push_str(&self.list_element_entries.list[self.weekday_selected].timings.timing_collection[self.timing_selected].clone().format()); 
                                        self.character_index = self.input.chars().count();
                                        self.current_screen = CurrentScreen::Edit
                                    } else {
                                        self.reverse_state();
                                    }
                                },
                                TimingOp::Exit => self.current_screen = CurrentScreen::Exit
                            };
                        };

                    }
                    _ => {}
                }
            },
            CurrentScreen::Add => {
                match key.code {
                    KeyCode::Esc => self.reverse_state(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Enter => {
                        self.previous_screen = CurrentScreen::Add;
                        if !self.timing_format_correct() {
                            self.error_type = ErrorType::Format;
                            self.current_screen = CurrentScreen::Error;
                        } else if !self.timing_format_no_clash() {
                            self.error_type = ErrorType::Clash;
                            self.current_screen = CurrentScreen::Error;
                        } else {
                            let t = self.parse_timing_from_input();
                            // add the new timing to the list
                            self.list_element_entries.list[self.weekday_selected].timings.timing_collection.push(t);
                            // TODO select the timing just created
                            // find the timing in the list
                            //let i = self.list_element_entries.list[self.weekday_selected].timings.timing_collection.iter().position(|t| <Timing as Clone>::clone(&t).format() == self.input).unwrap();
                            // set the selected timing on the widget wrapper
                            //self.timing_selected = i;

                            self.reverse_state();
                            self.input.clear();
                            self.character_index = 0;
                        }
                    }
                    _ => {}
                }
            },
            CurrentScreen::Edit => {
                match key.code {
                    KeyCode::Esc => self.reverse_state(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Enter => {
                        self.previous_screen = CurrentScreen::Edit;
                        // check, then parse the timing
                        if !self.timing_format_correct() {
                            self.error_type = ErrorType::Format;
                            self.current_screen = CurrentScreen::Error;
                        } else if !self.timing_format_no_clash() {
                            self.error_type = ErrorType::Clash;
                            self.current_screen = CurrentScreen::Error;
                        } else {
                            let t = self.parse_timing_from_input();
                            let _removed = std::mem::replace(&mut self.list_element_entries.list[self.weekday_selected].timings.timing_collection[self.timing_selected], t);
                            self.reverse_state();
                            self.input.clear();
                            self.character_index = 0;
                        }
                    }
                    _ => {}
                }
            },

            CurrentScreen::Delete => {
                match key.code {
                    KeyCode::Esc | KeyCode::Backspace => self.reverse_state(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        if let Some(i) = self.del_op_list.state.selected() {
                            match self.del_op_list.del_list[i].item.as_str() {
                                "Yes" => {
                                    if self.list_element_entries.list[self.weekday_selected].timings.timing_collection.len() > 0 {
                                        self.list_element_entries.list[self.weekday_selected].timings.timing_collection.remove(self.timing_selected);
                                    }
                                },
                                _ => {}
                            }
                        }
                        self.reverse_state();
                    }
                    _ => {}
                }
            },
            CurrentScreen::Error => {
                // use any key press to leave error screen
                match self.previous_screen {
                    CurrentScreen::Add => self.current_screen = CurrentScreen::Add,
                    CurrentScreen::Edit => self.current_screen = CurrentScreen::Edit,
                    _ => self.reverse_state()
                }
            },
            CurrentScreen::Exit => {
                match key.code {
                    KeyCode::Esc | KeyCode::Backspace => self.reverse_state(),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),

                    KeyCode::Char('h') | KeyCode::Left => self.reverse_state(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        // add code to select the list item
                        // render popup now using current selection
                        if let Some(i) = self.exit_list.state.selected() {
                            match self.exit_list.exit_list_items[i].item.as_str() {
                                "Yes" => {
                                    // compile the schedule here
                                    self.compile_schedule();                         
                                    self.should_exit = true;
                                },
                                _ => self.reverse_state()
                            }
                        }
                    },
                    _ => {}
                }

            }
        }
    }
    // this needs to extract the whole string and parse to two u32s that are the full times 
    // combined into one u32
    fn extract_timings_from_input(&self, input: &String) -> (u32, u32) {
        let two_timings = input.split("-").collect::<Vec<&str>>();

         let t_start = two_timings[0].split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap();
        let t_end = two_timings[1].split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap();

        (t_start, t_end)
    }

    fn timing_format_correct(&self) -> bool {
        let re = Regex::new(r"^(?<h>[0-2][0-9]):[0-5][0-9]:[0-5][0-9]-(?<h2>[0-2][0-9]):[0-5][0-9]:[0-5][0-9]$").unwrap();
        let re_matches = re.is_match(&self.input);
        if re_matches { 
            let times: Vec<(u32, u32)> = re.captures_iter(&self.input).map(|times| {
                let hour_1 = times.name("h").unwrap().as_str();
                let hour_1 = hour_1.parse::<u32>().unwrap();
                let hour_2 = times.name("h2").unwrap().as_str();
                let hour_2 = hour_2.parse::<u32>().unwrap();
                (hour_1, hour_2)
            }).collect();
            for time_pair in times.iter() {
                if time_pair.0 < 24 && 
                    time_pair.1 < 24 {
                    return true;
                } else {
                    return false;
                }
            }
            false
        } else {
            false
        }

    }

    fn timing_format_no_clash(&self) -> bool {
        // iterate through each timing for the current day
        // if input start is after any other start but before the end it will clash
        // Must not include current timing being edited
        let (start, end) = self.extract_timings_from_input(&self.input);
        for (i, t) in self.list_element_entries.list[self.weekday_selected].timings.timing_collection.iter().enumerate() {
            if self.current_screen == CurrentScreen::Edit && i == self.timing_selected { return true }
            let t_start = t.timing.0.split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap();
            let t_end = t.timing.1.split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap();
            if start <= t_start && end >= t_start {
                return false;
            } else if start <= t_start && end >= t_end {
                return false;
            } else if start <= t_end && end >= t_end {
                return false;
            } else if start >= t_start &&  end <= t_end {
                return false;
            }
        }
        true
    }

    fn parse_timing_from_input(&self) -> Timing {
        let new_t = self.input.as_str()
            .split("-")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let t = Timing::new(&new_t[0], &new_t[1]);
        t
    }
    
    fn reverse_state(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.current_screen = CurrentScreen::Weekdays,
            CurrentScreen::Day => self.current_screen = CurrentScreen::Weekdays,
            CurrentScreen::TimingOptions => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Add => self.current_screen = CurrentScreen::TimingOptions,
            CurrentScreen::Edit => self.current_screen = CurrentScreen::TimingOptions,
            CurrentScreen::Delete => self.current_screen = CurrentScreen::Day,
            CurrentScreen::Error => self.current_screen = CurrentScreen::TimingOptions,
            CurrentScreen::Exit => self.current_screen = CurrentScreen::Weekdays
        }
        self.previous_screen = self.current_screen.clone();
    }

    fn select_next(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_next(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_next(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_next(),
            CurrentScreen::Delete => self.del_op_list.state.select_next(),
            CurrentScreen::Exit => self.exit_list.state.select_next(),
            _ => {}
        }
    }
    fn select_previous(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_previous(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_previous(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_previous(),
            CurrentScreen::Delete => self.del_op_list.state.select_previous(),
            CurrentScreen::Exit => self.exit_list.state.select_previous(),
            _ => {}
        }

    }
    fn select_first(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_first(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_first(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_first(),
            CurrentScreen::Delete => self.del_op_list.state.select_first(),
            CurrentScreen::Exit => self.exit_list.state.select_first(),
            _ => {}

        }

    }
    fn select_last(&mut self) {
        match self.current_screen {
            CurrentScreen::Weekdays => self.list_element_entries.state.select_last(),
            CurrentScreen::Day => self.list_element_entries.list[self.weekday_selected].timings.state.select_last(),
            CurrentScreen::TimingOptions => self.timing_op_list.state.select_last(),
            CurrentScreen::Delete => self.del_op_list.state.select_last(),
            CurrentScreen::Exit => self.exit_list.state.select_last(),
            _ => {}
        }
    }

    // TODO cursor movement is not yet implemented due to having to set the position on the 
    // frame, and needed to pass the frame to the render function somehow
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {

            let current_index = self.character_index;
            let from_left_to_current_index = current_index -1;
            // get all characters before the selected character
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // get all characters after selected character
            let after_char_to_delete = self.input.chars().skip(current_index);

            //put all the chars together except the selected one (which is "deleted")
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Media Timer Setup")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.")
            .style(FOOTER_STYLE)
            .centered()
            .render(area, buf);
    }

    fn render_weekdays_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Select Day").centered())
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
            
        if self.list_element_entries.list[self.weekday_selected].timings.timing_collection.len() > 0 {
            let block = Block::new()
                .title(Line::raw("Edit Timings").centered())
                .borders(Borders::TOP | Borders::LEFT)
                .border_set(symbols::border::EMPTY)
                .border_style(ITEM_HEADER_STYLE)
                .bg(NORMAL_ROW_BG)
                .fg(TEXT_FG_COLOR);
            
            // sorting the Timing struct
            self.list_element_entries.list[self.weekday_selected]
                .timings.timing_collection.sort_by(|a, b| 
                    a.timing.0.split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap().cmp(&b.timing.0.split(":").collect::<Vec<&str>>().join("").parse::<u32>().unwrap()));

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
        } else {
            let input = Paragraph::new(Line::raw("No schedule set, press Enter to add a new one."))
               .style(SELECTED_STYLE)
               .bg(NORMAL_ROW_BG)
               .fg(TEXT_FG_COLOR)
               .wrap(Wrap { trim:true })
               .block(
                   Block::new()
                   .borders(Borders::TOP | Borders::LEFT)
                   .border_set(symbols::border::EMPTY)
                   .style(ITEM_HEADER_STYLE)
                   .title(Line::raw("Add Timing").centered())
               )
               .render(area, buf);

        }
    }

    fn render_op_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Select Task").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .timing_op_list
            .timing_ops
            .iter()
            .filter(|ops| 
                if &ops.op_item == "Edit" {
                    if self.list_element_entries.list[self.weekday_selected]
                            .timings.timing_collection.len() == 0 {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            )
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
    fn render_add(&self, area: Rect, buf: &mut Buffer) {
       let input = Paragraph::new(self.input.as_str()) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("Add Timing").centered())
           )
           .render(area, buf);
    }

    fn render_edit(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.

        let input = Paragraph::new(self.input.as_str()) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("Edit Timing").centered())
           )
           .render(area, buf);

    }

    fn render_delete(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        let block = Block::new()
            .title(Line::raw("Are you sure?").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .del_op_list
            .del_list
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let color = alternate_colors(i);
                ListItem::from(option).bg(color)
            })
            .collect();

        // create a list from all the items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.del_op_list.state);
    }
    fn render_error(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        let message = match self.error_type {
            ErrorType::Format => "Formating Error! Please check the timing format you have entered. Schedule timings must use the 24 hour clock and must follow the format 00:00:00-00:00:00",
            ErrorType::Clash => "Clash Error! Please check that the timing does not clash with another existing timing."
        };

        let input = Paragraph::new(Line::raw(message)) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .wrap(Wrap {trim:false})
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("ERROR").centered())
           )
           .render(area, buf);

    }
    fn render_exit(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        let block = Block::new()
            .title(Line::raw("Ready to exit the schedule?").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .exit_list
            .exit_list_items
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let color = alternate_colors(i);
                ListItem::from(option).bg(color)
            })
            .collect();

        // create a list from all the items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.exit_list.state);

    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // get the info
        /*
        let info = if let Some(i) = self.list_element_entries.state.selected() {
            self.list_element_entries.list[i].info.clone()
        } else {
            "Nothing selected...".to_string()
        };
        */
        let info = vec![ 
            Line::from("Use the arrow keys ⇅ to select a day."),
            Line::from("Use ENTER or → to display the schedule."),
            Line::from("Select a timing and press ENTER to Add, Edit or Delete."),
            Line::from("Enter the start and end timings for each new schedule entry."),
            Line::from("Use ESC or ← to exit."),
            Line::from("Schedule timings must use the 24 hour clock and must follow the format 00:00:00-00:00:00"),
            Line::from("Example: 12:20:00-13:15:00"),
        ];

        // show the list item's info under the list
        let block = Block::new()
            .title(Line::raw("INSTRUCTIONS").centered())
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

        Clear.render(area, buf);

        match self.current_screen {
            CurrentScreen::Weekdays => {
                let [header_area, main_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(area);

                let [list_area, item_area] = Layout::vertical([
                    Constraint::Fill(2),
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

                self.render_op_list(popup_area, buf);
           },
            CurrentScreen::Add => {
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
                // set the cursor area
                self.input_area = popup_area;
                self.render_add(popup_area, buf);

            },
            CurrentScreen::Edit => {
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
                // set the cursor area
                self.input_area = popup_area;
                self.render_edit(popup_area, buf);
            },
            CurrentScreen::Delete => {
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

                self.render_delete(popup_area, buf);
            },
            CurrentScreen::Error => {
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

                self.render_error(popup_area, buf);
            },

            CurrentScreen::Exit => {
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


                self.render_exit(popup_area, buf);

            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_reverse_state() {
        let mut t_widget = TimingsWidget::default();
        // current state is CurrentScreen::Weekdays
        t_widget.current_screen = CurrentScreen::Day;
        t_widget.reverse_state();
        // state should now be CurrentScreen::Weekdays
        
        assert!(t_widget.current_screen == CurrentScreen::Weekdays);

        // set state to CurrentScreen::TimingOptions
        t_widget.current_screen = CurrentScreen::TimingOptions;
        t_widget.reverse_state();

        assert!(t_widget.current_screen == CurrentScreen::Day);

        t_widget.current_screen = CurrentScreen::Add;
        t_widget.reverse_state();
        assert!(t_widget.current_screen == CurrentScreen::TimingOptions);

        t_widget.current_screen = CurrentScreen::Edit;
        t_widget.reverse_state();
        assert!(t_widget.current_screen == CurrentScreen::TimingOptions);

        t_widget.current_screen = CurrentScreen::Delete;
        t_widget.reverse_state();
        assert!(t_widget.current_screen == CurrentScreen::Day);

        t_widget.current_screen = CurrentScreen::Error;
        t_widget.reverse_state();
        assert!(t_widget.current_screen == CurrentScreen::TimingOptions);

        t_widget.current_screen = CurrentScreen::Exit;
        t_widget.reverse_state();
        assert!(t_widget.current_screen == CurrentScreen::Weekdays);
    }

    #[test]
    fn check_parse_timing_from_input() {
        let mut t_widget = TimingsWidget::default();
        t_widget.input = String::from("10:00:00-11:00:00");
        let timing = t_widget.parse_timing_from_input();
        assert_eq!(timing.timing.0, "10:00:00");
        assert_eq!(timing.timing.1, "11:00:00");
    }

    #[test]
    fn check_timing_format_no_clash() {
        let mut t_widget = TimingsWidget::default();
        // check timing in the middle
        t_widget.input = String::from("11:11:11-12:12:12");
        let no_clash = t_widget.timing_format_no_clash();
        // should be false because the default timing is 9am-5pm
        // there is a clash
        assert_eq!(no_clash, false);

        // check timing that overlaps with start
        t_widget.input = String::from("08:00:00-11:00:00");
        let no_clash = t_widget.timing_format_no_clash();
        // should be false because the default timing is 9am-5pm
        // there is a clash
        assert_eq!(no_clash, false);

        // check timing that overlaps with start and end
        t_widget.input = String::from("08:00:00-18:00:00");
        let no_clash = t_widget.timing_format_no_clash();
        // should be false because the default timing is 9am-5pm
        // there is a clash
        assert_eq!(no_clash, false);

        // check timing that overlaps with end
        t_widget.input = String::from("16:16:54-18:02:24");
        let no_clash = t_widget.timing_format_no_clash();
        // should be false because the default timing is 9am-5pm
        // there is a clash
        assert_eq!(no_clash, false);

        t_widget.input = String::from("08:00:00-08:50:00");
        let no_clash = t_widget.timing_format_no_clash();
        // should be no clash 
        assert_eq!(no_clash, true);

        t_widget.input = String::from("17:00:01-23:50:00");
        let no_clash = t_widget.timing_format_no_clash();
        // should be no clash 
        assert_eq!(no_clash, true);

_
    }

    #[test]
    fn check_timing_format_correct() {
        let mut t_widget = TimingsWidget::default();
        t_widget.input = String::from("10:00:00-11:00:00");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, true);

        t_widget.input = String::from("10:00:000-11:00:00");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, false);

        t_widget.input = String::from("10:00:00-11:000:00");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, false);

        t_widget.input = String::from("ten-four");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, false);

        t_widget.input = String::from("10:00-11:00");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, false);

        t_widget.input = String::from("some text here");
        let format_correct = t_widget.timing_format_correct();
        assert_eq!(format_correct, false);
    }

    #[test]
    fn check_extract_timings_from_input() {
        let mut t_widget = TimingsWidget::default();
        t_widget.input = String::from("10:00:00-11:00:00");
        let extracted_timings = t_widget.extract_timings_from_input(&t_widget.input);
        assert_eq!(extracted_timings.0, 100000 as u32);
        assert_eq!(extracted_timings.1, 110000 as u32);

        let input = String::from("12:12:12-16:00:00");
        let extracted_timings = t_widget.extract_timings_from_input(&input);
        assert_eq!(extracted_timings.0, 121212 as u32);
        assert_eq!(extracted_timings.1, 160000 as u32);


    }
}
