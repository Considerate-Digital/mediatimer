use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE, LIME},
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
use std::{
    error::Error,
    rc::Rc,
    cell::RefCell,
    path::{
    PathBuf,
    Path
    },
    process::Command,
};
use crate::ProcType;

use ratatui_explorer::{FileExplorer, Theme};

const ITEM_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const TEXT_DIR_COLOR: Color = GREEN.c200;


pub struct FileSelectWidget {
    should_exit: bool,
    file_explorer: FileExplorer,
    selected_file: FileSelect,
}

type FileSelect = PathBuf;

impl Default for FileSelectWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: Path::new("./").to_path_buf(),
        }
    }
}

impl FileSelectWidget {
    /*
    pub fn new() -> Self {
        Self {
            should_exit: false,
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: Path::new("./").to_path_buf(),
        }
    }
    */

    pub fn run (mut self, mut terminal: &mut DefaultTerminal) -> Result<FileSelect, Box< dyn Error>> {
        self.setup_file_explorer();
        self.style_file_explorer();

        while !self.should_exit {
            terminal.draw(|f| f.render_widget(&mut self, f.area()))?;
            let event = event::read()?;
            
            self.file_explorer.handle(&event)?;

            if let Event::Key(key) = event {
                self.handle_key(key);
            };

        }
        Ok(self.selected_file)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let is_dir = self.file_explorer.current().is_dir();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_exit = true;
            }
            KeyCode::Enter if is_dir == false => {
                // check that it is not "../" or "./"
                println!("{:?}", self.file_explorer.current().name());

                self.set_current_type();
                self.should_exit = true;
            }
            _ => {}
        }
    }
    
    fn set_current_type(&mut self) {
        self.selected_file = self.file_explorer.current().path().to_path_buf();
    }
    /*
    fn find_likely_usb() -> Option<&Path> {
       // use this function to identify and examine a likely usb mount 
       // first test if any usbs have been mounted using the medialoop_init program
       // then check for any usb mount points
    }
    */

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

    fn style_file_explorer(&mut self) {
        let theme = Theme::default()
            .add_default_title()
            .with_block(Block::default()
                .title(Line::raw("Select a media file.").centered())
                .borders(Borders::TOP)
                .border_set(symbols::border::EMPTY)
                .border_style(ITEM_HEADER_STYLE)
                .bg(NORMAL_ROW_BG)
                .padding(Padding::horizontal(1))
                )
            .with_highlight_item_style(SELECTED_STYLE)
            .with_highlight_symbol("> ".into())
            .with_highlight_spacing(HighlightSpacing::Always)
            .with_dir_style(Style::default().fg(TEXT_DIR_COLOR))
            .with_highlight_dir_style(SELECTED_STYLE)
            .with_item_style(Style::default().fg(TEXT_FG_COLOR));
        
        self.file_explorer.set_theme(theme)
    }

    fn setup_file_explorer(&mut self) {
        /* if let Some(usb) = find_likely_usb() {
         *      self.file_explorer.set_cwd(usb.to_str()).unwrap();
         * } else {
         *      self.file_explorer.set_cwd("./").unwrap();
         * }
         */
    

        // temporary
        self.file_explorer.set_cwd("/mnt").unwrap();


    }
    
    fn render_file_explorer(&mut self, area: Rect, buf: &mut Buffer) {
        self.file_explorer.widget().render(area, buf);

    }
    /*
    fn render_file_(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Find and select your media file").centered())
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
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        // we have to diferentiate this "render" from the render fn on self
        StatefulWidget::render(list, area, buf, &mut self.proc_type_entries.state);
    }
    */

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // get the info
        let text = vec![ 
            Line::from("Select a media file to loop using our file explorer."),
            Line::from("Use the keyboard arrows and the 'Enter' key to find the file you want to loop."),
            Line::from("Press the 'Enter' key to select the file."),
            Line::from("You can exit the file explorer at any time by pressing 'ESC' or 'q'."),

        ];

        // show the list item's info under the list
        let block = Block::new()
            .title(Line::raw("TYPE INFO").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // now render the item info
        Paragraph::new(text)
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

impl Widget for &mut FileSelectWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [file_area, item_area] = Layout::vertical([
            Constraint::Fill(3),
            Constraint::Fill(1)
        ])
        .areas(main_area);

        FileSelectWidget::render_header(header_area, buf);
        FileSelectWidget::render_footer(footer_area, buf);
        self.render_file_explorer(file_area, buf);
        self.render_selected_item(item_area, buf);
    }

}

