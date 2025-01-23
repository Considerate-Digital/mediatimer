use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap, ListItem, List,
        HighlightSpacing
    },
    DefaultTerminal,
};
use std::error::Error;

const ITEM_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;

use crate::Autoloop;

pub struct AutoloopWidget {
    should_exit: bool,
    selected_type: Autoloop,
    list_element_entries: AutoloopList
}

struct AutoloopList {
    list: Vec<AutoloopEntry>,
    state: ListState
}

impl FromIterator<(Autoloop, &'static str)> for AutoloopList {
    fn from_iter<I: IntoIterator<Item = (Autoloop, &'static str)>>(iter: I) -> Self {
        let list = iter
            .into_iter()
            .map(|(list_element, info)| AutoloopEntry::new(list_element, info))
            .collect();
        let state = ListState::default();
        Self { list, state }
    }
}
struct AutoloopEntry {
    list_element: Autoloop,
    info: String,
}

impl From<&AutoloopEntry> for ListItem<'_> {
    fn from(value: &AutoloopEntry) -> Self {
        let line = Line::styled(format!("{}", value.list_element.to_string()), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

impl AutoloopEntry {
    fn new(list_element: Autoloop, info: &str) -> Self {
        Self {
            list_element,
            info: info.to_string()
        }
    }
}

impl Default for AutoloopWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
            selected_type: Autoloop::No,
            list_element_entries: AutoloopList::from_iter([
                (Autoloop::Yes, "Auto loop this media file."),
                (Autoloop::No, "Do not auto loop this media file.")

            ]),
        }
    }
}

impl AutoloopWidget {
    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<Autoloop, Box< dyn Error>> {
        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(self.selected_type)
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
                self.set_current_type();
                self.should_exit = true;
            }
            _ => {}
        }
    }
    
    fn set_current_type(&mut self) {
        if let Some(i) = self.list_element_entries.state.selected() {
            match self.list_element_entries.list[i].list_element {
                Autoloop::Yes => self.selected_type = Autoloop::Yes,
                Autoloop::No => self.selected_type = Autoloop::No,
            }
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
        Paragraph::new("Medialoop Setup")
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
            .title(Line::raw("Do you want your file to automatically loop?").centered())
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

impl Widget for &mut AutoloopWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        AutoloopWidget::render_header(header_area, buf);
        AutoloopWidget::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }

}

