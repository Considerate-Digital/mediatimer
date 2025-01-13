use clokwerk::{
    Scheduler, TimeUnits, Job,
    Interval::*
};
use std::{
    fmt,
    fs,
    io,
    io::Write,
    io::Error as IoError,
    thread,
    time::Duration,
    path::{
        Path,
        PathBuf,
    },
    env,
};
use std::error::Error;
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

use home::home_dir;

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


#[derive(Debug, Display, PartialEq)]
enum ProcType {
    Media,
    Browser,
    Executable,
}

#[derive(Debug, Display)]
enum Autoloop {
    Yes,
    No
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
    auto_loop: Autoloop,
    timings: Timings,
    file: PathBuf
}

impl Task {
    fn new(proc_type: ProcType, auto_loop: Autoloop, timings: Timings, file: PathBuf) -> Self {
        Task {
            proc_type,
            auto_loop,
            timings,
            file
        }
    }
    fn set_loop(&mut self, auto_loop: Autoloop) {
        self.auto_loop = auto_loop;
    }
    fn set_proc_type(&mut self, p_type: ProcType) {
        self.proc_type = p_type;
    }
    fn set_weekday(&mut self, wd: Weekday) {
        self.timings.push(wd);
    }
}

fn write_task(task: Task) -> Result<(), IoError> {
   if let Some(dir) = home::home_dir() {
        // check if dir exists
        let mut dir_path = PathBuf::from(dir);
        dir_path.push("medialoop_config");

        // check if the medialoop directory exists in home
        if dir_path.as_path().is_dir() == false {
            // create the medialoop directory if it does not exist
            if let Err(er) = fs::create_dir(dir_path.as_path()) {
                eprintln!("Directory could not be created");
               IoError::other("Could not create medialoop directory.");
            }
        }

        // write task to .env file in medialoop directory
        dir_path.push("vars");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&dir_path)?;
    
       // write proctype
       writeln!(file, "ML_PROCTYPE={}", task.proc_type.to_string().to_lowercase())?;

       //write autoloop
        writeln!(file, "ML_AUTOLOOP={}", match task.auto_loop {
            Autoloop::Yes => "true",
            Autoloop::No => "false"
        });

       // write timings
       // create print each day as one env var and separate timings using " ".
       // format is START-STOP e.g. 0900-1500
       /*
       for timing in task.timings.iter() {
          let day_times_fmt = timing.iter().map(|i| format!("{}-{}", i.0, i.1).collect();
           if let Err(e) = writeln!(file, "ML_{}={}", timing.to_string().to_uppercase(), day_times_fmt.join(,)) {
               eprintln!("Could not write to file: {}", e);
           }
       }
       */
       

       // write file
       writeln!(file, "ML_FILE={}", task.file.display())?;
            

   } else {
       eprintln!("Could not find home directory.");
       IoError::other("Could not find home directory");
   }
   Ok(())
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
    let proctype = ProcTypeWidget::default().run(terminal)?;

    let mut terminal = ratatui::init();
    // return Ok(FileSelectType)
    let file_path = FileSelectWidget::default().run(terminal)?;
    
    let mut terminal = ratatui::init();
    // return Ok(Autoloop) e.g. Ok(Autoloop::No)
    let mut autoloop = Autoloop::No;
    if proctype == ProcType::Media {
        autoloop = AutoloopWidget::default().run(terminal)?;
    }

    let mut terminal = ratatui::init();

    //returns Ok(Timings)
    //let timings = TimingsWidget::default().run(terminal)?;

    let mut terminal = ratatui::init();

    // if the selected file is on a usb stick
    // edit fstab to automount that usb
    let timings = Vec::new();
    let task = Task::new(proctype, autoloop, timings, file_path);

    // write a function that writes the task to a specific env file
    // write_task 
    if let Err(e) = write_task(task) {
        eprintln!("Error writing tasks to env file: {}", e);
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
