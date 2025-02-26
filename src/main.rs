use std::{
    fs,
    io,
    io::Write,
    io::Error as IoError,
    path::{
        PathBuf,
    },
    process::{
        Command
    }
};
use std::error::Error;
use ratatui::{
    prelude::CrosstermBackend,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use strum::Display;

mod proctype;
use crate::proctype::ProcTypeWidget;

mod fileselect;
use crate::fileselect::FileSelectWidget;

mod autoloop;
use crate::autoloop::AutoloopWidget;

mod advanced_schedule;
use crate::advanced_schedule::AdvancedScheduleWidget;

mod landing;
use crate::landing::LandingWidget;

mod reboot;
use crate::reboot::RebootWidget;

mod mount;

mod styles;

mod timings;

use crate::timings::{
    TimingsWidget
};

#[derive(Debug, Display, PartialEq)]
pub enum ProcType {
    Media,
    Browser,
    Executable,
}

#[derive(Debug, Display)]
pub enum Autoloop {
    Yes,
    No
}

#[derive(Debug, Display, PartialEq)]
pub enum AdvancedSchedule {
    Yes,
    No
}


#[derive(Debug, Display, PartialEq)]
pub enum Reboot {
    Yes,
    No
}

type Schedule = Vec<(String, String)>;
type Timings = Vec<Weekday>;

#[derive(Display, Debug)]
pub enum Weekday {
    Monday(Schedule),
    Tuesday(Schedule),
    Wednesday(Schedule),
    Thursday(Schedule),
    Friday(Schedule),
    Saturday(Schedule),
    Sunday(Schedule),
}

impl Weekday {
    fn as_str(&self) -> &'static str {
        match self {
            Weekday::Monday(_) => "Monday",
            Weekday::Tuesday(_) => "Tuesday",
            Weekday::Wednesday(_) => "Wednesday",
            Weekday::Thursday(_) => "Thursday",
            Weekday::Friday(_) => "Friday",
            Weekday::Saturday(_) => "Saturday",
            Weekday::Sunday(_) => "Sunday"
        }
    }
    fn to_string(&self) -> String {
        match self {
            Weekday::Monday(_) => String::from("Monday"),
            Weekday::Tuesday(_) => String::from("Tuesday"),
            Weekday::Wednesday(_) => String::from("Wednesday"),
            Weekday::Thursday(_) => String::from("Thursday"),
            Weekday::Friday(_) => String::from("Friday"),
            Weekday::Saturday(_) => String::from("Saturday"),
            Weekday::Sunday(_) => String::from("Sunday")
        }
    }

    fn timings(&self) -> Schedule {
        match self {
            Weekday::Monday(schedule) => schedule.clone(),
            Weekday::Tuesday(schedule) => schedule.clone(),
            Weekday::Wednesday(schedule) => schedule.clone(),
            Weekday::Thursday(schedule) => schedule.clone(),
            Weekday::Friday(schedule) => schedule.clone(),
            Weekday::Saturday(schedule) => schedule.clone(),
            Weekday::Sunday(schedule) => schedule.clone()
        }
    }
}

fn default_timings() -> Timings {
    vec![
        Weekday::Monday(Vec::new()),
        Weekday::Tuesday(Vec::new()),
        Weekday::Wednesday(Vec::new()),
        Weekday::Thursday(Vec::new()),
        Weekday::Friday(Vec::new()),
        Weekday::Saturday(Vec::new()),
        Weekday::Sunday(Vec::new()),
    ]
}


/// This program runs one task at custom intervals. The task can also be looped.
/// Commonly this is used for playing media files at certain times.
/// The Task struct is the main set of instructions that are written out into an env file to be 
/// interpreted in future by the init program.

#[derive( Debug)]
struct Task {
    proc_type: ProcType,
    auto_loop: Autoloop,
    advanced_schedule: AdvancedSchedule,
    timings: Timings,
    file: PathBuf
}

impl Task {
    fn new(proc_type: ProcType, auto_loop: Autoloop, advanced_schedule: AdvancedSchedule, timings: Timings, file: PathBuf) -> Self {
        Task {
            proc_type,
            auto_loop,
            advanced_schedule,
            timings,
            file
        }
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
               eprintln!("Directory could not be created: {}", er);
               IoError::other("Could not create medialoop directory.");
            }
        }

        // write task to .env file in medialoop directory
        dir_path.push("vars");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&dir_path)?;
    
       // write proctype
       let _ = writeln!(file, "ML_PROCTYPE=\"{}\"", task.proc_type.to_string().to_lowercase())?;

       //write autoloop
       let _ = writeln!(file, "ML_AUTOLOOP=\"{}\"", match task.auto_loop {
            Autoloop::Yes => "true",
            Autoloop::No => "false"
        });
       let _ = writeln!(file, "ML_SCHEDULE=\"{}\"", match task.advanced_schedule {
            AdvancedSchedule::Yes => "true",
            AdvancedSchedule::No => "false"
       });

       // TODO write timings
       // create print each day as one env var and separate timings using " ".
       // format is START-STOP e.g. 0900-1500
       //
       // This function should be converted to a closure
       fn format_print_day_schedule(day: String, schedule: Schedule, mut file: fs::File) {
           let day_times_fmt: Vec<String> = schedule.iter().map(|i| format!("{}-{}", i.0, i.1)).collect();
           if let Err(e) = writeln!(file, "ML_{}={}", day.to_uppercase(), day_times_fmt.join(",")) {
               eprintln!("Could not write to file: {}", e);
           }
       }
       for timing in task.timings.iter() {
           if let Ok(file_clone) = file.try_clone() {
               match timing {
                   Weekday::Monday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Tuesday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Wednesday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Thursday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Friday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Saturday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   Weekday::Sunday(schedule) => format_print_day_schedule(timing.to_string(), schedule.to_vec(), file_clone),
                   _ => {}
              }
           }
       }
       

       // write file
       let _ = writeln!(file, "ML_FILE=\"{}\"", task.file.display())?;
        /*
       // advanced use
       let _ = writeln!(file, "# Change this to 'true' if you want to use a custom schedule");
       let _ = writeln!(file, "ML_SCHEDULE=\"false\"");

       //full schedule layout
       let schedule = "#ML_MONDAY=\"09:00-12:00,13:00-17:00\"\n#ML_TUESDAY=\"09:00-12:00,13:00-17:00\"\n#ML_WEDNESDAY=\"09:00-12:00,13:00-17:00\"\n#ML_THURSDAY=\"09:00-12:00,13:00-17:00\"\n#ML_FRIDAY=\"09:00-12:00,13:00-17:00\"\n#ML_SATURDAY=\"09:00-12:00,13:00-17:00\"\n#ML_SUNDAY=\"09:00-12:00,13:00-17:00\"\n";
       let _ = writeln!(file, "# Remove the '#' at the start of each day that you require a customised schedule for.\n# Edit the timings and add new entries if needed.\n# Make sure the timings have the format START-END and are comma (',') separated with no spaces.\n# Schedule timings can be specified in either minute-format (10:00) or second-format (10:00:00)\n# Note that the auto-loop feature only applies to media files and you must implement internal loops yourself for browser-based or executable files.");
       let _ = writeln!(file, "{}", schedule);
        */
            

   } else {
       eprintln!("Could not find home directory.");
       IoError::other("Could not find home directory");
   }
   Ok(())
}

/// Before the program starts, it unmounts and remounts any usb drives.
/// This is  called in order to unmount and remount any usbs using the naming conventions
/// that the medialoop_init uses. The mount points for usb drives must be standardised in
/// order for this program to work. The program is designed this way so that it can be utilised 
/// by non-technical users.

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
    
    // temporarily remove mounting capabilities
    //let _usb_drive_mount = find_mount_drives()?;


    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let _backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::init();

    let _landing = LandingWidget::default().run(&mut terminal)?;
    // returns Ok(ProcType) e.g. Ok(ProcType::Media)
    let proctype = ProcTypeWidget::default().run(&mut terminal)?;

    // return Ok(FileSelectType)
    let file_path = FileSelectWidget::default().run(&mut terminal)?;
    
    // return Ok(Autoloop) e.g. Ok(Autoloop::No)
    let mut autoloop = Autoloop::No;
    if proctype == ProcType::Media {
        autoloop = AutoloopWidget::default().run(&mut terminal)?;
    }

    let advanced_schedule = AdvancedScheduleWidget::default().run(&mut terminal)?;

    let mut timings = default_timings();
    if advanced_schedule == AdvancedSchedule::Yes {
        //returns Ok(Timings)
        timings = TimingsWidget::default().run(&mut terminal)?;
    }

    let task = Task::new(proctype, autoloop, advanced_schedule, timings, file_path);

    // write a function that writes the task to a specific env file
    // write_task 
    if let Err(e) = write_task(task) {
        eprintln!("Error writing tasks to env file: {}", e);
    }


    // return Ok(Reboot) e.g. Ok(Reboot::No)
    let reboot = RebootWidget::default().run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if reboot selected then reboot
        match reboot {
            Reboot::Yes => {
                let _reboot = Command::new("reboot")
                    .output()
                    .expect("could not reboot");
            }
            Reboot::No => {}
        }

    Ok(())
}
