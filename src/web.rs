use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect, Position},
    style::{
        Color, Stylize, Style, Modifier
    },
    symbols,
    text::{Line},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Clear,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};

use std::{
    path::{
        PathBuf,
        Path
    },
    error::Error,
    io::{BufRead, BufReader},
    fs
};

use log::{
    info
};
use crate::{
    logi,
    loge,
    logw,
};

/*
use wl_clipboard_rs::{
    paste::{
        get_contents,
        ClipboardType,
        Error as ClipboardError,
        MimeType,
        Seat
    }
};
*/

use crate::areas;
use regex::Regex;

use crate::styles::{
    ITEM_HEADER_STYLE,
    NORMAL_ROW_BG,
    ALT_ROW_BG_COLOR,
    SELECTED_STYLE,
    TEXT_FG_COLOR,
    TEXT_DIR_COLOR,
    FOOTER_STYLE
};

type FileSelect = PathBuf;
use ratatui_explorer::{FileExplorer, Theme};
// This is declared twice due to the TUI list structure requirements and must be converted 
// between main and this module

#[derive(Debug, Clone, PartialEq)]
enum CurrentScreen {
    Menu,
    Add,
    Import,
    Message,
    Error,
    Exit
}

#[derive(Clone)]
enum Menu {
    Add,
    Import,
    Exit
}

impl Menu {
    fn as_vec_of_str(&self) -> Vec<MenuItem> {
        vec![
            MenuItem::from("Add"), 
            MenuItem::from("Import"),
            MenuItem::from("Exit")
        ]
    }
}

struct MenuList {
    menu_ops: Vec<MenuItem>,
    state: ListState
}

impl MenuList {
    fn default() -> MenuList {
        let mut state = ListState::default();
        state.select_first();
        MenuList {
            menu_ops: Menu::Add.as_vec_of_str(),
            state
        }
    }
}

struct MenuItem {
    op_item: String
}

impl MenuItem {
    fn from(new: &str) -> MenuItem {
        MenuItem {
            op_item: String::from(new)
        }
    }
}

impl From<&MenuItem> for ListItem<'_> {
    fn from(value: &MenuItem) -> Self {
        let line = Line::styled(value.op_item.clone(), TEXT_FG_COLOR);
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
        let line = Line::styled(value.item.clone(), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

enum ErrorType {
    Format
}

pub struct WebWidget {
    message_text: String,
    file_explorer: FileExplorer,
    selected_file: FileSelect,
    should_exit: bool,
    current_screen: CurrentScreen,
    previous_screen: CurrentScreen,
    // selected weekdays and timings are indexes starting at 0
    url: String,
    operation_selected: Menu,
    menu_op_list: MenuList,
    input: String,
    character_index: usize,
    input_area: Rect,
    exit_list: ExitList,
    error_type: ErrorType,
    mounted_drives: Vec<(PathBuf, String)>
}

impl Default for WebWidget {
    // Only used for tests
    fn default() -> Self {
        Self {
            message_text: String::new(),
            file_explorer: FileExplorer::new().unwrap(),
            selected_file: PathBuf::new(),
            should_exit: false,
            current_screen: CurrentScreen::Menu,
            previous_screen: CurrentScreen::Menu,
            url: String::new(),
            operation_selected: Menu::Add,
            menu_op_list: MenuList::default(),
            input: String::new(),
            character_index: 0,
            input_area: Rect::new(0,0,0,0),
            exit_list: ExitList::default(),
            error_type: ErrorType::Format,
            mounted_drives: Vec::new()
        }
    }
}
impl WebWidget {
    pub fn new (new_url: String, mounted_drives: Vec<(PathBuf, String)>) -> Result<Self, Box<dyn Error>> {
        let file_explorer = FileExplorer::new()?;
        Ok(
            Self {
                message_text: String::new(),
                file_explorer,
                selected_file: Path::new("./").to_path_buf(),
                should_exit: false,
                current_screen: CurrentScreen::Menu,
                previous_screen: CurrentScreen::Menu,
                url: new_url.clone(),
                operation_selected: Menu::Add,
                menu_op_list: MenuList::default(),
                input: new_url.clone(),
                character_index: 0,
                input_area: Rect::new(0,0,0,0),
                exit_list: ExitList::default(),
                error_type: ErrorType::Format,
                mounted_drives
            }
        )
    }
    pub fn run (mut self, terminal: &mut DefaultTerminal) -> Result<String, Box< dyn Error>> {

        let _file_explorer_init = self.setup_file_explorer()?;
        let _file_explorer_style_init = self.style_file_explorer();

        while !self.should_exit {
            terminal.draw(|f| {
                f.render_widget(&mut self, f.area());
                    if self.current_screen == CurrentScreen::Add {
                        f.set_cursor_position(Position::new(
                                self.input_area.x + self.character_index as u16 + 1,
                                // move one line down, from the border to the input lin
                                self.input_area.y + 1,
                        ))
                }
            })?;

            let event = event::read()?;
                if self.current_screen == CurrentScreen::Import {
                        let _ = self.file_explorer.handle(&event);
                }
                if let Event::Key(key) = event {
                    let _handle = self.handle_key(key)?;
                    //self.text_area.input(key);
                }
        }
        Ok(self.url)
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
    fn setup_file_explorer(&mut self) -> Result<(), Box<dyn Error>> {
        let username = whoami::username();

        if self.selected_file.to_str() != Some("") && self.selected_file.is_file() {
            if let Some(parent_dir) = self.selected_file.parent() {
                self.file_explorer.set_cwd(parent_dir)?;
                // then highlight the selected file
                if let Some(file_os_str) = self.selected_file.file_name() && let Some(file_name) = file_os_str.to_str() {
                        let files = self.file_explorer.files();
                        if let Some(index) = files.iter().position(|f| f.name() == file_name) {
                            self.file_explorer.set_selected_idx(index);
                        }
                }
            } else {
                self.file_explorer.set_cwd("/home/")?;
            }
        } else if self.mounted_drives.len() > 1 && !username.is_empty() {
            let path_buf: PathBuf = ["/media/", &username].iter().collect();
            self.file_explorer.set_cwd(&path_buf)?;
        } else if self.mounted_drives.len() == 1 {
            self.file_explorer.set_cwd(self.mounted_drives[0].0.clone())?;
        } else if !username.is_empty() {
            let username = whoami::username();
            let path_buf: PathBuf = ["/home/", &username].iter().collect();
            self.file_explorer.set_cwd(path_buf)?;
        } else {
            self.file_explorer.set_cwd("/home/")?;
        }
        Ok(())
    }
    fn handle_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match self.current_screen {
            CurrentScreen::Menu => {
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
                        if let Some(i) = self.menu_op_list.state.selected() {
                            
                            let op = match i {
                                0 => Menu::Add,
                                1 => Menu::Import,
                                2 => Menu::Exit,
                                _ => Menu::Add

                            };

                            self.operation_selected = op.clone();

                            match op {
                                Menu::Add => self.current_screen = CurrentScreen::Add,
                                Menu::Import => {
                                    self.current_screen = CurrentScreen::Import;
                                },
                                Menu::Exit => self.current_screen = CurrentScreen::Exit
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
                    /* TODO clipboard not working
                    KeyCode::Char('v') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                        //paste the contents into input
                            let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
                            match result {
                                Ok((mut pipe, _)) => {
                                    let mut contents = vec![];
                                    pipe.read_to_end(&mut contents).unwrap();
                            }

                            Err(ClipboardError::NoSeats) | Err(ClipboardError::ClipboardEmpty) | Err(ClipboardError::NoMimeType) => {
                                // The clipboard is empty or doesn't contain text, nothing to worry about.
                            }

                            Err(err) => Err(err).unwrap(),
                            }
                        }
                    },
                    */
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Enter => {
                        self.previous_screen = CurrentScreen::Add;
                        self.clean_input();
                        let url_format = self.url_format_correct()?;
                        if !url_format {
                            self.error_type = ErrorType::Format;
                            self.current_screen = CurrentScreen::Error;
                            logi!("Url format incorrect");
                        } else {
                            self.url = self.input.clone();
                            self.message_text = format!("Url selected: {}", &self.url);
                            self.current_screen = CurrentScreen::Message;
                            self.character_index = 0;
                        }
                    }
                    _ => {}
                }
            },
           CurrentScreen::Error => {
                // use any key press to leave error screen
                match self.previous_screen {
                    CurrentScreen::Add => self.current_screen = CurrentScreen::Add,
                    CurrentScreen::Import => self.current_screen = CurrentScreen::Menu,
                    _ => self.reverse_state()
                }
            },
            CurrentScreen::Message => {
                // use any key press to leave message screen
                match self.previous_screen {
                    CurrentScreen::Add => self.current_screen = CurrentScreen::Exit,
                    CurrentScreen::Import => self.current_screen = CurrentScreen::Exit,
                    _ => self.reverse_state()
                }
            },
            CurrentScreen::Import => {
                let is_dir = self.file_explorer.current().is_dir();
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace | KeyCode::Esc => self.reverse_state(),
                    KeyCode::Enter if !is_dir => {
                        let current_path_buf = self.file_explorer.current().path().to_path_buf();
                        self.selected_file = current_path_buf;
                        let file = fs::File::open(&self.selected_file)?;
                        let reader = BufReader::new(file);
                        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;


                        self.input = lines[0].clone();
                        self.clean_input();
                        let format_correct = self.url_format_correct()?;
                        if !format_correct {
                            self.error_type = ErrorType::Format;
                            self.current_screen = CurrentScreen::Error;
                            logi!("Url format incorrect");
                        } else {
                            // add the new timing to the list
                            self.url = self.input.clone();

                            self.reverse_state();
                            self.character_index = 0;
                        }
                        self.message_text = format!("Url imported: {}", &self.url);
                        self.current_screen = CurrentScreen::Message;
                        },
                    _ => {}
                }

            },
            CurrentScreen::Exit => {
                match key.code {
                    KeyCode::Char('m') => self.current_screen = CurrentScreen::Menu,
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
        Ok(())
    }
    fn clean_input(&mut self) {
        self.input = String::from(self.input.trim());
    }
    fn url_format_correct(&self) -> Result<bool, Box<dyn Error>> {
        let re = Regex::new(r"^(https?://)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.-]*)*\/?$")?;
        Ok(re.is_match(&self.input))
    }

    
    fn reverse_state(&mut self) {
        match self.current_screen {
            CurrentScreen::Menu => self.current_screen = CurrentScreen::Exit,
            _ => self.current_screen = CurrentScreen::Menu
        }
        /*
        self.current_screen = CurrentScreen::Menu;
        */
        self.previous_screen = self.current_screen.clone();
    }
    fn select_next(&mut self) {
        match self.current_screen {
            CurrentScreen::Menu => self.menu_op_list.state.select_next(),
            CurrentScreen::Exit => self.exit_list.state.select_next(),
            _ => {}
        }
    }
    fn select_previous(&mut self) {
        match self.current_screen {
            CurrentScreen::Menu => self.menu_op_list.state.select_previous(),
            CurrentScreen::Exit => self.exit_list.state.select_previous(),
            _ => {}
        }

    }
    fn select_first(&mut self) {
        match self.current_screen {
            CurrentScreen::Menu => self.menu_op_list.state.select_first(),
            CurrentScreen::Exit => self.exit_list.state.select_first(),
            _ => {}

        }

    }
    fn select_last(&mut self) {
        match self.current_screen {
            CurrentScreen::Menu => self.menu_op_list.state.select_last(),
            CurrentScreen::Exit => self.exit_list.state.select_last(),
            _ => {}
        }
    }

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

    // rendering logic
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Web")
            .bold()
            .centered()
            .render(area, buf);
    }
    
    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        let mut text = Vec::with_capacity(20);

        match self.current_screen {
            CurrentScreen::Menu => {
                text = vec![ 
                    Line::from("Select \"Add\" to type or paste a URL from the browser."),
                    Line::from("Select \"Import\" to import a URL directly from a text file."),
                    Line::from("Select \"Exit\" to leave this screen."),
                ];

            },
            CurrentScreen::Add => {
                text = vec![
                    Line::from("Type the web URL or use Shift+Ctl+v to paste a URL from the browser. The URL must match the format provided in your browser and start with https://")
                ];
            },
            CurrentScreen::Import => {
                text = vec![
                    Line::from("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.")
                ];
            },
            CurrentScreen::Exit => {
                text = vec![
                    Line::from("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.")
                ];
            },
            _ => {}
        }
        // show the list item's info under the list
        let block = Block::new()
            .title(Line::raw("INFO").centered().style(FOOTER_STYLE))
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
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let mut text = String::with_capacity(20);

        match self.current_screen {
            CurrentScreen::Menu => {
                text = String::from("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.");
            },
            CurrentScreen::Add => {
                text = String::from("Type the web URL or use Shift+Ctl+v to paste a URL from the browser.");
            },
            CurrentScreen::Import => {
                text = String::from("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.");
            },
            CurrentScreen::Exit => {
                text = String::from("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom, and ESC to exit.");
            },
            _ => {}
        }

        Paragraph::new(text.as_str())
            .style(FOOTER_STYLE)
            .centered()
            .render(area, buf);
    }

    fn render_menu_op_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Select Task").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all the timings in the weekday selected and stylise them
        let items: Vec<ListItem> = self 
            .menu_op_list
            .menu_ops
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
        StatefulWidget::render(list, area, buf, &mut self.menu_op_list.state);

    }
    fn render_add(&self, area: Rect, buf: &mut Buffer) {
       Paragraph::new(self.input.as_str()) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("Add Url").centered())
           )
           .render(area, buf);
    }

   fn render_file_explorer(&mut self, area: Rect, buf: &mut Buffer) {
        self.file_explorer.widget().render(area, buf);
    }
    fn render_file_explorer_selected_item(&self, area: Rect, buf: &mut Buffer) {
            let text = vec![ 
                Line::from("Select a text file containing a URL using our file explorer."),
                Line::from("Use the arrow keys ⇅ to find the file you want to use."),
                Line::from("Press ENTER to select the file."),
                Line::from("To ascend a directory navigate to \"↑ Parent Folder ↑\" and press Enter"),
                Line::from("USB sticks will show up automatically. Manually find them in the directory '/media'."),
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

    fn render_file_explorer_header(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Select a file containing a URL")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_file_explorer_footer(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .style(FOOTER_STYLE)
            .centered()
            .render(area, buf);
    }


    fn render_message(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.

        Paragraph::new(Line::raw(&self.message_text)) 
           .fg(TEXT_FG_COLOR)
           .bg(NORMAL_ROW_BG)
           .wrap(Wrap {trim:false})
           .block(
               Block::bordered()
               .style(ITEM_HEADER_STYLE)
               .title(Line::raw("MESSAGE").centered())
           )
           .render(area, buf);

    }
    fn render_error(&mut self, area: Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        let message = match self.error_type {
            ErrorType::Format => "Formating Error! Please check the URL format. Note that URLs must start with \"https://\".",
        };

        Paragraph::new(Line::raw(message)) 
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
            .title(Line::raw("Ready to exit this stage?").centered())
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

    fn render_background(&self, area:Rect, buf: &mut Buffer) {
        // set the current input as the entry selected.
        Block::new()
            .title(Line::raw("URL setup").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(ITEM_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .render(area, buf);
    }
    /*
    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let info = vec![ 
            Line::from("Use the arrow keys ⇅ to select a day or press 'm' to open the menu."),
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
    */

}

const fn alternate_colors(i: usize) -> Color {
    if i.is_multiple_of(2) {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl Widget for &mut WebWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let popup_area: Rect = areas::popup_area(area);

        Clear.render(area, buf);

        match self.current_screen {
            CurrentScreen::Menu => {
                let [header_area, main_area, info_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(1),
                ])
                .areas(area);
                

                WebWidget::render_background(self, main_area, buf);
                WebWidget::render_header(header_area, buf);
                WebWidget::render_footer(self, footer_area, buf);
                WebWidget::render_info(self, info_area, buf);
                
                Clear.render(popup_area, buf);
                self.render_menu_op_list(popup_area, buf);
           },
           CurrentScreen::Add => {
                let [header_area, main_area, info_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(1),
                ])
                .areas(area);
                

                WebWidget::render_background(self, main_area, buf);
                WebWidget::render_header(header_area, buf);
                WebWidget::render_footer(self, footer_area, buf);
                WebWidget::render_info(self, info_area, buf);
                Clear.render(popup_area, buf);
                // set the cursor area
                self.input_area = popup_area;
                self.render_add(popup_area, buf);

            },
            CurrentScreen::Import => {
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

                self.render_file_explorer_header(header_area, buf);
                self.render_file_explorer_footer(footer_area, buf);
                self.render_file_explorer(file_area, buf);
                self.render_file_explorer_selected_item(item_area, buf);
            },
            CurrentScreen::Message => {
                let [header_area, main_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(area);

                WebWidget::render_background(self, main_area, buf);
                WebWidget::render_header(header_area, buf);
                WebWidget::render_footer(self, footer_area, buf);

                Clear.render(popup_area, buf);
                self.render_message(popup_area, buf);
            },

            CurrentScreen::Error => {
                let [header_area, main_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(area);

                WebWidget::render_background(self, main_area, buf);
                WebWidget::render_header(header_area, buf);
                WebWidget::render_footer(self, footer_area, buf);

                Clear.render(popup_area, buf);
                self.render_error(popup_area, buf);
            },

            CurrentScreen::Exit => {
                let [header_area, main_area, footer_area] = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(area);

                WebWidget::render_background(self, main_area, buf);
                WebWidget::render_header(header_area, buf);
                WebWidget::render_footer(self, footer_area, buf);

                Clear.render(popup_area, buf);

                self.render_exit(popup_area, buf);

            },
        }
    }
}

