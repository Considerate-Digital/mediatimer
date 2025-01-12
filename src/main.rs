use clokwerk::{
    Scheduler, TimeUnits, Job,
    Interval::*
};
use std::{
    fmt,
    io,
    thread,
    time::Duration,
    path::{
        Path,
        PathBuf,
    },
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

mod fileselect;
use crate::fileselect::FileSelectWidget;

mod autoloop;
use crate::autoloop::AutoloopWidget;

/*
mod timings;
use crate::timings::TimingsWidget;
*/


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
struct Task {
    proc_type: ProcType,
    auto_loop: bool,
    timings: Timings,
    file: PathBuf
}

impl Task {
    fn new(proc_type: ProcType, auto_loop: bool, timings: Timings, file: PathBuf) -> Self {
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



    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::init();

    // returns Ok(ProcType) e.g. Ok(ProcType::Media)
    let proctype = ProcTypeWidget::default().run(terminal).unwrap();

    let mut terminal = ratatui::init();
    // return Ok(FileSelectType)
    let mut file_path = FileSelectWidget::default().run(terminal).unwrap();
    
    let mut terminal = ratatui::init();
/*
    println!("File: {}, Dir: {}", current_file.name(), current_dir.display());
*/
    // if the selected file is on a usb stick
    // edit fstab to automount that usb
    let timings = Vec::new(); 
    let task = Task::new(proctype, true, timings, file_path);


    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("{:?}", task);

    Ok(())
}
