use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        Color, Stylize,
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

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    ALT_ROW_BG_COLOR,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
};
use crate::Reboot;

pub struct RebootWidget {
    should_exit: bool,
    selected_type: Reboot,
    list_element_entries: RebootList
}

struct RebootList {
    list: Vec<RebootEntry>,
    state: ListState
}

impl FromIterator<(Reboot, &'static str)> for RebootList {
    fn from_iter<I: IntoIterator<Item = (Reboot, &'static str)>>(iter: I) -> Self {
        let list = iter
            .into_iter()
            .map(|(list_element, info)| RebootEntry::new(list_element, info))
            .collect();
        let state = ListState::default();
        Self { list, state }
    }
}
struct RebootEntry {
    list_element: Reboot,
    info: String,
}

impl From<&RebootEntry> for ListItem<'_> {
    fn from(value: &RebootEntry) -> Self {
        let line = Line::styled(format!("{}", value.list_element.to_string()), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

impl RebootEntry {
    fn new(list_element: Reboot, info: &str) -> Self {
        Self {
            list_element,
            info: info.to_string()
        }
    }
}

impl Default for RebootWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
            selected_type: Reboot::No,
            list_element_entries: RebootList::from_iter([
                (Reboot::Yes, "Reboot my computer now."),
                (Reboot::No, "Do not reboot my computer now, I will do it later.")

            ]),
        }
    }
}

impl RebootWidget {
    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<Reboot, Box< dyn Error>> {
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
                Reboot::Yes => self.selected_type = Reboot::Yes,
                Reboot::No => self.selected_type = Reboot::No,
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
            .title(Line::raw("Reboot now?").centered())
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

impl Widget for &mut RebootWidget {
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

        RebootWidget::render_header(header_area, buf);
        RebootWidget::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }

}

