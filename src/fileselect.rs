use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        Modifier,
        Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, Padding, Paragraph,
        Widget, Wrap,
    },
    DefaultTerminal,
};
use std::{
    error::Error,
    path::{
    PathBuf,
    Path
    },
    fs::File
};

use ratatui_explorer::{FileExplorer, Theme};

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
    TEXT_DIR_COLOR,
    FOOTER_STYLE
};

use crate::mount::identify_mounted_drives;


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
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            should_exit: false,
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: file_path,
        }
    }

    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<FileSelect, Box< dyn Error>> {
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

                self.set_current_type();
                self.should_exit = true;
            }
            _ => {}
        }
    }
    
    fn set_current_type(&mut self) {
        self.selected_file = self.file_explorer.current().path().to_path_buf();
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

    fn style_file_explorer(&mut self) {
        let theme = Theme::default()
            .add_default_title()
            .with_block(Block::default()
                .title(Line::raw("Select File").centered())
                .borders(Borders::TOP)
                .border_set(symbols::border::EMPTY)
                .border_style(ITEM_HEADER_STYLE)
                .bg(NORMAL_ROW_BG)
                .padding(Padding::horizontal(1))
                )
            .with_highlight_item_style(SELECTED_STYLE)
            .with_highlight_symbol("> ".into())
            .with_highlight_spacing(HighlightSpacing::Always)
            .with_dir_style(Style::default().fg(TEXT_DIR_COLOR).add_modifier(Modifier::BOLD))
            .with_highlight_dir_style(SELECTED_STYLE)
            .with_item_style(Style::default().fg(TEXT_FG_COLOR));
        
        self.file_explorer.set_theme(theme)
    }

    fn setup_file_explorer(&mut self) {
        let mounted_drives = identify_mounted_drives();
        let username = whoami::username();

        if self.selected_file.to_str() != Some("") && self.selected_file.is_file() {
            if let Some(parent_dir) = self.selected_file.parent() {
                self.file_explorer.set_cwd(&parent_dir).unwrap();
                // then highlight the selected file
                if let Some(file_os_str) = self.selected_file.file_name() { 
                    if let Some(file_name) = file_os_str.to_str() {
                        let files = self.file_explorer.files();
                        if let Some(index) = files.iter().position(|f| f.name() == file_name) {
                            self.file_explorer.set_selected_idx(index);
                        }
                    }
                }
            } else {
                self.file_explorer.set_cwd("/home/").unwrap();
            }
        } else if mounted_drives.len() > 1 && !username.is_empty() {
            let path_buf: PathBuf = ["/media/", &username].iter().collect();
            self.file_explorer.set_cwd(&path_buf).unwrap();
        } else if mounted_drives.len() == 1 {
            self.file_explorer.set_cwd(&mounted_drives[0]).unwrap();
        } else if !username.is_empty() {
            let username = whoami::username();
            let path_buf: PathBuf = ["/home/", &username].iter().collect();
            self.file_explorer.set_cwd(path_buf).unwrap();
        } else {
            self.file_explorer.set_cwd("/home/").unwrap();
        }
    }
    
    fn render_file_explorer(&mut self, area: Rect, buf: &mut Buffer) {
        self.file_explorer.widget().render(area, buf);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // get the info
        let text = vec![ 
            Line::from("Select a file using our file explorer."),
            Line::from("Use the arrow keys ⇅ to find the file you want to use."),
            Line::from("Press ENTER to select the file."),
            Line::from("To ascend a directory navigate to \"↑ Parent Folder ↑\" and press Enter"),
            Line::from("USB sticks will show up automatically. Manually find them in the directory 'media'."),

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
        Paragraph::new(text)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
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

