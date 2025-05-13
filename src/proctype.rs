use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        Color,  Stylize,
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

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    ALT_ROW_BG_COLOR,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
    FOOTER_STYLE
};

pub struct ProcTypeWidget {
    should_exit: bool,
    selected_type: ProcType,
    proc_type_entries: ProcTypeList
}

struct ProcTypeList {
    list: Vec<ProcTypeEntry>,
    state: ListState
}

impl FromIterator<(ProcType, &'static str)> for ProcTypeList {
    fn from_iter<I: IntoIterator<Item = (ProcType, &'static str)>>(iter: I) -> Self {
        let list = iter
            .into_iter()
            .map(|(proc_type, info)| ProcTypeEntry::new(proc_type, info))
            .collect();
        let mut state = ListState::default();
        state.select_first();
        Self { list, state }
    }
}
struct ProcTypeEntry {
    proc_type: ProcType,
    info: String,
}

impl From<&ProcTypeEntry> for ListItem<'_> {
    fn from(value: &ProcTypeEntry) -> Self {
        let line = Line::styled(format!("{}", value.proc_type.to_string()), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

impl ProcTypeEntry {
    fn new(proc_type: ProcType, info: &str) -> Self {
        Self {
            proc_type,
            info: info.to_string()
        }
    }
}

impl ProcTypeWidget {

    #[cfg(feature="eco")]
    pub fn new(preset_type: ProcType) -> Self {
        Self {
            should_exit: false,
            selected_type: preset_type,
            proc_type_entries: ProcTypeList::from_iter([
                (ProcType::Video, "A video file, played without audio. Example files: mp4, avi, mkv etc. Most formats are accepted."),
                (ProcType::Audio, "An audio file. Example files: mp3, wav, flac etc. Most formats are accepted."),
                (ProcType::Image, "An image file. Example files: jpg, png, webp etc. Most formats are accepted."),
                (ProcType::Slideshow, "A slideshow of images. Example files: jpg, png, webp etc. Most formats are accepted. The folder selected must only contain images."),
                (ProcType::Browser, "A browser based application or file, such as P5 or html."),
                (ProcType::Executable, "A binary executable or shell script. Use this option to launch complex software installations via a shell script."),

            ]),
        }
    }
    
    #[cfg(feature="standard")]
    pub fn new(preset_type: ProcType) -> Self {
        Self {
            should_exit: false,
            selected_type: preset_type,
            proc_type_entries: ProcTypeList::from_iter([
                (ProcType::Video, "A video file. Example files: mp4, avi, mkv etc. Most formats are accepted."),
                (ProcType::Audio, "An audio file. Example files: mp3, wav, flac etc. Most formats are accepted."),
                (ProcType::Image, "An image file. Example files: jpg, png, webp etc. Most formats are accepted."),
                (ProcType::Slideshow, "A slideshow of images. Example files: jpg, png, webp etc. Most formats are accepted. The folder selected must only contain images."),
                (ProcType::Browser, "A browser based application or file, such as P5 or html."),
                (ProcType::Executable, "A binary executable or shell script. Use this option to launch complex software installations via a shell script."),

            ]),
        }
    }

    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<ProcType, Box< dyn Error>> {
        // set default value
        // get the index of the selected item
        let index = self.proc_type_entries.list.iter().position(|i| i.proc_type == self.selected_type).unwrap();
        // set the state
        self.proc_type_entries.state.select(Some(index));

        while !self.should_exit {
            let _ = &terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
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
        if let Some(i) = self.proc_type_entries.state.selected() {
            match self.proc_type_entries.list[i].proc_type {
                ProcType::Video => self.selected_type = ProcType::Video,
                ProcType::Audio => self.selected_type = ProcType::Audio,
                ProcType::Image => self.selected_type = ProcType::Image,
                ProcType::Slideshow => self.selected_type = ProcType::Slideshow,
                ProcType::Browser => self.selected_type = ProcType::Browser,
                ProcType::Executable => self.selected_type = ProcType::Executable,
            }
        }
    }

    fn select_none(&mut self) {
        self.proc_type_entries.state.select(None);
    }
    fn select_next(&mut self) {
        self.proc_type_entries.state.select_next();
    }
    fn select_previous(&mut self) {
        self.proc_type_entries.state.select_previous();
    }
    fn select_first(&mut self) {
        self.proc_type_entries.state.select_first();
    }
    fn select_last(&mut self) {
        self.proc_type_entries.state.select_last();
    }


    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Media Timer Setup")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .style(FOOTER_STYLE)
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
            .proc_type_entries
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
        StatefulWidget::render(list, area, buf, &mut self.proc_type_entries.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // get the info
        let (info, mut title) = if let Some(i) = self.proc_type_entries.state.selected() {
            let pt_str = match self.proc_type_entries.list[i].proc_type {
                ProcType::Video => ProcType::Video.as_ref(),
                ProcType::Audio => ProcType::Audio.as_ref(),
                ProcType::Image => ProcType::Image.as_ref(),
                ProcType::Slideshow => ProcType::Slideshow.as_ref(),
                ProcType::Browser => ProcType::Browser.as_ref(),
                ProcType::Executable => ProcType::Executable.as_ref(),
            };
            let title_str = String::from(pt_str.to_uppercase());
                (self.proc_type_entries.list[i].info.clone(), title_str)
             } else {
                 ("Nothing selected...".to_string(), "".to_string())
        };
    

        title.push_str(" INFO");

        // show the list item's info under the list
        let block = Block::new()
            .title(Line::raw(&title).centered().style(FOOTER_STYLE))
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

impl Widget for &mut ProcTypeWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        ProcTypeWidget::render_header(header_area, buf);
        ProcTypeWidget::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }

}

