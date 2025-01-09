use clokwerk::{
    Scheduler, TimeUnits, Job,
    Interval::*
};
use std::{
    fmt,
    io,
    thread,
    time::Duration,
    path::Path,
    env,
    error::Error
};
use ratatui::{
    prelude::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Borders, Block, Paragraph},
    Frame,
};
use crossterm::{
    event::{self, Event, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui_explorer::{FileExplorer, Theme};
use strum::Display;

mod proctype;
use crate::proctype::ProcTypeWidget;

#[derive(Debug, Display)]
enum ProcType {
    Media,
    Browser,
    Executable,
    Java
}

#[derive( Debug)]
enum Weekday {
    Monday(Vec<(u32, u32)>),
    Tuesday(Vec<(u32, u32)>),
    Wednesday(Vec<(u32, u32)>),
    Thursday(Vec<(u32, u32)>),
    Friday(Vec<(u32, u32)>),
    Saturday(Vec<(u32, u32)>),
    Sunday(Vec<(u32, u32)>),
}

type Timings = Vec<Weekday>;

/// This program runs one task at custom intervals. The task can also be looped.
/// Commonly this is used for playing media files at certain times.
/// The Task struct is the main set of instructions that are written out into an env file to be 
/// interpreted in future by the init program.
#[derive( Debug)]
struct Task<'a> {
    proc_type: ProcType,
    auto_loop: bool,
    timings: Timings,
    file: &'a Path
}

impl <'a> Task<'a> {
    fn new(proc_type: ProcType, auto_loop: bool, timings: Timings, file: &'a Path) -> Self {
        Task {
            proc_type,
            auto_loop,
            timings,
            file
        }
    }
    fn set_loop(&mut self, auto_loop: bool) {
        self.auto_loop = auto_loop;
    }
    fn set_proc_type(&mut self, p_type: ProcType) {
        self.proc_type = p_type;
    }
    fn set_weekday(&mut self, wd: Weekday) {
        self.timings.push(wd);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // useful for getting current settings
    // use this dir .env for testing
    /*
    dotenvy::from_path(Path::new("/home/alex/medialoop/src/.env"))?;

    for (key, value) in env::vars() {
        match key.as_str() {
            "ML_WEEKDAYS" => println!("{}", value),
            "ML_START" => println!("{}", value),
            "ML_END" => println!("{}", value),
            _ => {}
        }
    }
    */

    //grep -iE '/dev/sd(a|b|c|d)' /proc/mounts
    //
    //
    // check for usb devices
    // mount all usb devices /mnt/sda /mnt/sdb etc.
    

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::init();

    let theme = Theme::default().add_default_title();
    let mut file_explorer = FileExplorer::with_theme(theme)?;

    // set the current dir of the file explorer -- identify and prefer a usb
    file_explorer.set_cwd("/home/alex/").unwrap();
    
    let proctype_selection = ProcTypeWidget::default().run(terminal);
    println!("{:?}", proctype_selection);
    

    let mut terminal = ratatui::init();
    loop {
            terminal.draw(|f| {
                let size = f.size();
                let block = Block::default()
                    .title(
                        Span::styled("MediaLoop", 
                            Style::default()
                                .fg(Color::Yellow)
                                .bg(Color::Blue)
                        )
                    );
                let text = vec![ 
                    Line::from("Select a media file to loop using our file explorer."),
                    Line::from("Use the keyboard arrows and the 'Enter' key to find the file you want to loop."),
                    Line::from("Press the 'Enter' key to select the file."),
                    Line::from("You can exit the file explorer at any time by pressing 'ESC' or 'q'."),

                    Line::from(""),
                    Line::from(""),
                    Line::from("Now press SPACE to continue."),
                ];
                let paragraph = Paragraph::new(text).block(block);
                f.render_widget(paragraph, size);
            });
        let event = event::read();
        if let Ok(Event::Key(key)) = event {
            let key_code = key.code;
            match key_code {
                KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => break,
                _ => {}
            }
        }
   
    }
    
    let mut selected_file = String::with_capacity(20);

    loop {
        terminal.draw(|f| {
            f.render_widget(&file_explorer.widget(), f.area());
        })?;
        let event = event::read();
        if let Ok(Event::Key(key)) = event {
            let key_code = key.code;
            let current_is_dir = file_explorer.current().is_dir(); 
            match key_code {
                KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Enter if current_is_dir == false => {
                    selected_file = file_explorer.current().path().display().to_string();
                    break;
                },
                _ => {}
            }
        }
        file_explorer.handle(&event?);
    }
    
/*
    let current_file = file_explorer.current();
    let current_dir = file_explorer.cwd();
    println!("File: {}, Dir: {}", current_file.name(), current_dir.display());
*/
    println!("selected file: {}", selected_file);
    // if the selected file is on a usb stick
    // edit fstab to automount that usb
     

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
