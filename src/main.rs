use std::{
    env,
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
use regex::Regex;
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
mod areas;

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

#[derive(Debug, Display, PartialEq)]
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

#[derive(Display, Debug, Clone)]
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

fn to_weekday(value: String, day: Weekday) -> Result<Weekday, Box<dyn Error>> {
    let string_vec: Vec<String> = value.as_str().split(",").map(|x| x.trim().to_string()).collect(); 
    if &value != "" {
        let string_vec_test = string_vec.clone();

        // check the schedule format matches 00:00 or 00:00:00
        // move these check to the "to weekday" function
        let re = Regex::new(r"(^\d{2}:\d{2}-\d{2}:\d{2}$|^\d{2}:\d{2}:\d{2}-\d{2}:\d{2}:\d{2}$|^\d{2}:\d{2}-\d{2}:\d{2}:\d{2}$|^\d{2}:\d{2}:\d{2}-\d{2}:\d{2}$)").unwrap();
        // check the times split correctly
        let parsed_count = string_vec_test.len();  
        let string_of_times = string_vec_test.iter().map(|s| s.to_string()).collect::<String>();
        let mut re_count = 0;
        for time in string_vec_test.iter() {
            if re.is_match(&time) == true {
                re_count += 1;
            }
        }
        if parsed_count != re_count {
            // timings do not match
            eprintln!("Schedule incorrectly formatted!");
        }
    }

    let mut day_schedule = Vec::new();
    for time in string_vec.iter() {
        let start_end = time.as_str()
            .split("-")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        if start_end.len() == 2 {
            let start = start_end[0].clone();
            let end = start_end[1].clone();
            day_schedule.push((start, end));
        }
    }
    match day {
       Weekday::Monday(_) =>  Ok(Weekday::Monday(day_schedule)),
       Weekday::Tuesday(_) => Ok(Weekday::Tuesday(day_schedule)),
       Weekday::Wednesday(_) => Ok(Weekday::Wednesday(day_schedule)),
       Weekday::Thursday(_) => Ok(Weekday::Thursday(day_schedule)),
       Weekday::Friday(_) => Ok(Weekday::Friday(day_schedule)),
       Weekday::Saturday(_) => Ok(Weekday::Saturday(day_schedule)),
       Weekday::Sunday(_) => Ok(Weekday::Sunday(day_schedule))
    }


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
        dir_path.push(".medialoop_config");

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

    // issue command to pause medialoop_init
    // systemctl --user stop medialoop_init.service
    let _stop_medialoop_init = Command::new("systemctl")
        .arg("--user")
        .arg("stop")
        .arg("medialoop_init.service")
        .output()
        .expect("Medialoop not restarted");


    // Find and load any existing config for the user
    // This is hard coded, as the user will always be named "fun"
    let username = whoami::username();
    let env_dir_path: PathBuf =["/home/", &username, ".medialoop_config/vars"].iter().collect();

    


    // set up the config vars
    let mut file = PathBuf::new();
    let mut proc_type = ProcType::Media;
    let mut auto_loop = Autoloop::Yes;
    let mut schedule = AdvancedSchedule::No;
    let mut timings: Timings = Vec::with_capacity(7);
    let mut monday: Weekday = Weekday::Monday(Vec::with_capacity(2));
    let mut tuesday: Weekday = Weekday::Tuesday(Vec::with_capacity(2));
    let mut wednesday: Weekday = Weekday::Wednesday(Vec::with_capacity(2));
    let mut thursday: Weekday = Weekday::Thursday(Vec::with_capacity(2));
    let mut friday: Weekday = Weekday::Friday(Vec::with_capacity(2));
    let mut saturday: Weekday = Weekday::Saturday(Vec::with_capacity(2));
    let mut sunday: Weekday = Weekday::Sunday(Vec::with_capacity(2));
    
    if env_dir_path.exists() {
        if let Err(e) = dotenvy::from_path_override(env_dir_path.as_path()) {
            eprintln!("Cannot find env vars at path: {}", env_dir_path.display());
        }
        // parse the environmental vars
        for (key, value) in env::vars() {
            match key.as_str() {
                // proctype should always be stored and checked lowercase
                "ML_PROCTYPE" => proc_type = match value.to_lowercase().as_str() {
                    "media" => ProcType::Media,
                    "browser" => ProcType::Browser,
                    "executable" => ProcType::Executable,
                    &_ => ProcType::Media
                },
                "ML_AUTOLOOP" => auto_loop = match value.as_str() {
                    "true" => Autoloop::Yes,
                    "false" => Autoloop::No,
                    &_ => Autoloop::No
                },
                "ML_FILE" => file.push(value.as_str()),
                "ML_SCHEDULE" => schedule = match value.as_str() {
                    "true" => AdvancedSchedule::Yes,
                    "false" => AdvancedSchedule::No,
                    &_ => AdvancedSchedule::No
                },
                "ML_MONDAY" => monday = to_weekday(value, Weekday::Monday(Vec::new()))?,
                "ML_TUESDAY" => tuesday = to_weekday(value, Weekday::Tuesday(Vec::new()))?,
                "ML_WEDNESDAY" => wednesday = to_weekday(value, Weekday::Wednesday(Vec::new()))?,
                "ML_THURSDAY" => thursday = to_weekday(value, Weekday::Thursday(Vec::new()))?,
                "ML_FRIDAY" => friday = to_weekday(value, Weekday::Friday(Vec::new()))?,
                "ML_SATURDAY" => saturday = to_weekday(value, Weekday::Saturday(Vec::new()))?,
                "ML_SUNDAY" => sunday = to_weekday(value, Weekday::Sunday(Vec::new()))?,
                _ => {}
            }
        }

        timings = vec![monday, tuesday, wednesday, thursday, friday, saturday, sunday]; 
    }

    let timings_clone = timings.clone();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let _backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::init();

    let _landing = LandingWidget::default().run(&mut terminal)?;
    // returns Ok(ProcType) e.g. Ok(ProcType::Media)
    let proctype = ProcTypeWidget::new(proc_type).run(&mut terminal)?;

    // return Ok(FileSelectType)
    let file_path = FileSelectWidget::new(file).run(&mut terminal)?;
    
    let advanced_schedule = AdvancedScheduleWidget::new(schedule).run(&mut terminal)?;

    if advanced_schedule == AdvancedSchedule::Yes {
        //returns Ok(Timings)
        timings = TimingsWidget::new(timings).run(&mut terminal)?;
    }
    
    // return Ok(Autoloop) e.g. Ok(Autoloop::No)
    if proctype == ProcType::Media && advanced_schedule == AdvancedSchedule::Yes {
        auto_loop = AutoloopWidget::new(auto_loop).run(&mut terminal)?;
    }
    let task = Task::new(proctype, auto_loop, advanced_schedule, timings, file_path);

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

    // issue command to restart medialoop_init service
    let _enable_medialoop_init = Command::new("systemctl")
        .arg("--user")
        .arg("start")
        .arg("medialoop_init.service")
        .output()
        .expect("Medialoop not restarted");

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
