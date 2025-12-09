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
        Widget, Wrap, Clear
    },
    DefaultTerminal,
};
use std::{
    error::Error,
    path::{
    PathBuf,
    Path
    },
    process::Command
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

use crate::ProcType;
use crate::areas;

use regex::Regex;

pub struct FileSelectWidget {
    should_exit: bool,
    file_explorer: FileExplorer,
    selected_file: FileSelect,
    can_be_dir: bool,
    proc_type: ProcType,
    mounted_drives: Vec<(PathBuf, String)>,
    error: bool,
    error_message: String
}

type FileSelect = PathBuf;

impl Default for FileSelectWidget {
    fn default() -> Self {
        Self {
            should_exit: false,
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: Path::new("./").to_path_buf(),
            can_be_dir: false,
            proc_type: ProcType::Video,
            mounted_drives: Vec::new(),
            error: false,
            error_message: String::from("")
        }
    }
}

impl FileSelectWidget {
    pub fn new(file_path: PathBuf, can_be_dir: bool, proc_type: ProcType, mounted_drives: Vec<(PathBuf, String)>) -> Self {
        Self {
            should_exit: false,
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: file_path,
            can_be_dir,
            proc_type,
            mounted_drives,
            error: false,
            error_message: String::from("")
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
            KeyCode::Enter if !is_dir => {
                if self.error {
                    self.error = false;
                } else if self.can_be_dir {
                    let mut current_path_buf = self.file_explorer.current().path().to_path_buf();
                    current_path_buf.pop();
                    self.selected_file = current_path_buf;
                    self.should_exit = true;
                } else if self.proc_type == ProcType::Video {
                        if self.video_compatible() {
                            self.selected_file = self.file_explorer.current().path().to_path_buf();
                            self.should_exit = true;
                        }
                    } else {
                        self.selected_file = self.file_explorer.current().path().to_path_buf();
                        self.should_exit = true;
                    }
            }
            _ => {}
        }
    }
        
    // check for video size
    #[cfg(feature="eco")]
    fn video_compatible(&mut self) -> bool {
        let file_path = self.file_explorer.current().path();
        let vid_height = Command::new("ffprobe")
            .arg("-loglevel")
            .arg("error")
            .arg("-show_streams")
            .arg(&file_path)
            .output()
            .expect("height in pixels");

        let height_string = String::from_utf8_lossy(&vid_height.stdout);
        let height_re = Regex::new(r"height=(?<height>\d+)").unwrap();
        if let Some(height_info) = height_re.captures(&height_string) {
            if let height_collected = height_info[1].to_string() {
                let height_int: u32 = height_collected.parse::<u32>().unwrap();
                if height_int > 1080 {
                    self.error_message = format!("Video resolution is too high ({}). Export video as 1080p (HD) maximum.", height_int);
                    self.error = true;
                    return false;
                } else {
                    return true;
                }
                
            }
        } else {
            self.error_message = format!("Video resolution could not be identified. Please check file is a supported video format.");
            self.error = true;
        }
        return false;
    }

    // check for video size
    #[cfg(feature="standard")]
    fn video_compatible(&mut self) -> bool {
        let file_path = self.file_explorer.current().path();
        let vid_height = Command::new("ffprobe")
            .arg("-loglevel")
            .arg("error")
            .arg("-show_streams")
            .arg(&file_path)
            .output()
            .expect("height in pixels");

        let height_string = String::from_utf8_lossy(&vid_height.stdout);
        let height_re = Regex::new(r"height=(?<height>\d+)").unwrap();
        if let Some(height_info) = height_re.captures(&height_string) {
            if let height_collected = height_info[1].to_string() {
                let height_int: u32 = height_collected.parse::<u32>().unwrap();
                if height_int > 1440 {
                    self.error_message = format!("Video resolution is too high ({}). Export video as 1440p (QHD) maximum.", height_int);
                    self.error = true;
                    return false;
                } else {
                    return true;
                }
                
            }
        } else {
            self.error_message = format!("Video resolution could not be identified. Please check file is a supported video format.");
            self.error = true;
        }
        return false;
    }

    // check for video size
    #[cfg(feature="pro")]
    fn video_compatible(&mut self) -> bool {
        let file_path = self.file_explorer.current().path();
        let vid_height = Command::new("ffprobe")
            .arg("-loglevel")
            .arg("error")
            .arg("-show_streams")
            .arg(file_path)
            .output()
            .expect("height in pixels");

        let height_string = String::from_utf8_lossy(&vid_height.stdout);
        let height_re = Regex::new(r"height=(?<height>\d+)").unwrap();
        if let Some(height_info) = height_re.captures(&height_string) {
            let height_collected = height_info[1].to_string();
                let height_int: u32 = height_collected.parse::<u32>().unwrap();
                if height_int > 2160 {
                    self.error_message = format!("Video resolution is too high ({}). Export video as 2160p (4K) maximum.", height_int);
                    self.error = true;
                    return false;
                } else {
                    return true;
                }
                
        } else {
            self.error_message = "Video resolution could not be identified. Please check file is a supported video format.".to_string();
            self.error = true;
        }
        false
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

    fn render_error(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        let popup_area: Rect = areas::popup_area(area);
        Paragraph::new(Line::raw(""))
            .bg(NORMAL_ROW_BG)
            .block(
                Block::new()
            )
            .render(area, buf);

        Paragraph::new(Line::raw(&self.error_message)) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .wrap(Wrap {trim:false})
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("ERROR").centered())
           )
           .render(popup_area, buf);
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
            .with_highlight_symbol("> ")
            .with_highlight_spacing(HighlightSpacing::Always)
            .with_dir_style(Style::default().fg(TEXT_DIR_COLOR).add_modifier(Modifier::BOLD))
            .with_highlight_dir_style(SELECTED_STYLE)
            .with_item_style(Style::default().fg(TEXT_FG_COLOR));
        
        self.file_explorer.set_theme(theme)
    }

    fn setup_file_explorer(&mut self) {
        let username = whoami::username();

        if self.selected_file.to_str() != Some("") && self.selected_file.is_file() {
            if let Some(parent_dir) = self.selected_file.parent() {
                self.file_explorer.set_cwd(parent_dir).unwrap();
                // then highlight the selected file
                if let Some(file_os_str) = self.selected_file.file_name() && let Some(file_name) = file_os_str.to_str() {
                        let files = self.file_explorer.files();
                        if let Some(index) = files.iter().position(|f| f.name() == file_name) {
                            self.file_explorer.set_selected_idx(index);
                        }
                }
            } else {
                self.file_explorer.set_cwd("/home/").unwrap();
            }
        } else if self.mounted_drives.len() > 1 && !username.is_empty() {
            let path_buf: PathBuf = ["/media/", &username].iter().collect();
            self.file_explorer.set_cwd(&path_buf).unwrap();
        } else if self.mounted_drives.len() == 1 {
            self.file_explorer.set_cwd(self.mounted_drives[0].0.clone()).unwrap();
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

        let mut text = vec![ 
                Line::from("Select a file using our file explorer."),
                Line::from("Use the arrow keys ⇅ to find the file you want to use."),
                Line::from("Press ENTER to select the file."),
                Line::from("To ascend a directory navigate to \"↑ Parent Folder ↑\" and press Enter"),
                Line::from("USB sticks will show up automatically. Manually find them in the directory 'media'."),

            ];

    
        if self.can_be_dir {
            text = vec![ 
                Line::from("Select a folder using our explorer."),
                Line::from("The target folder must not contain other folders."),
                Line::from("Use the arrow keys ⇅ to find the folder you want to use."),
                Line::from("Press ENTER to open the folder, navigate to one of your images and then press ENTER again."),
                Line::from("To ascend a directory navigate to \"↑ Parent Folder ↑\" and press Enter"),
                Line::from("USB sticks will show up automatically. Manually find them in the directory 'media'."),

            ];
        }
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

        if self.error {
            let popup_area: Rect = areas::popup_area(area);
            Clear.render(popup_area, buf);
            self.render_error(main_area, buf);
        } else {
            self.render_file_explorer(file_area, buf);
            self.render_selected_item(item_area, buf);
        }


    }

}

