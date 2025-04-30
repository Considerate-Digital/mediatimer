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
use strum::{
    Display,
    AsRefStr
};

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

mod loading;
use crate::loading::LoadingWidget;

mod reboot;
use crate::reboot::RebootWidget;

mod mount;

mod styles;
mod areas;

mod timings;

use crate::timings::{
    TimingsWidget
};

#[derive(Debug, Display, PartialEq, AsRefStr)]
pub enum ProcType {
    Video,
    Audio,
    Image,
    Slideshow,
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

#[derive(Display, Debug, Clone, PartialEq)]
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
    file: PathBuf,
    slide_delay: u32,
}

impl Task {
    fn new(proc_type: ProcType, auto_loop: Autoloop, advanced_schedule: AdvancedSchedule, timings: Timings, file: PathBuf, slide_delay: u32) -> Self {
        Task {
            proc_type,
            auto_loop,
            advanced_schedule,
            timings,
            file,
            slide_delay,
        }
    }
}

fn write_task(task: Task) -> Result<(), IoError> {
   if let Some(dir) = home::home_dir() {
        // check if dir exists
        let mut dir_path = PathBuf::from(dir);
        dir_path.push(".mediatimer_config");

        // check if the mediatimer directory exists in home
        if dir_path.as_path().is_dir() == false {
            // create the mediatimer directory if it does not exist
            if let Err(er) = fs::create_dir(dir_path.as_path()) {
               eprintln!("Directory could not be created: {}", er);
               IoError::other("Could not create mediatimer directory.");
            }
        }

        // write task to .env file in mediatimer directory
        dir_path.push("vars");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&dir_path)?;
    
       // write proctype
       let _ = writeln!(file, "MT_PROCTYPE=\"{}\"", task.proc_type.to_string().to_lowercase())?;

       //write autoloop
       let _ = writeln!(file, "MT_AUTOLOOP=\"{}\"", match task.auto_loop {
            Autoloop::Yes => "true",
            Autoloop::No => "false"
        });
       let _ = writeln!(file, "MT_SCHEDULE=\"{}\"", match task.advanced_schedule {
            AdvancedSchedule::Yes => "true",
            AdvancedSchedule::No => "false"
       });

       // This function should be converted to a closure
       fn format_print_day_schedule(day: String, schedule: Schedule, mut file: fs::File) {
           let day_times_fmt: Vec<String> = schedule.iter().map(|i| format!("{}-{}", i.0, i.1)).collect();
           if let Err(e) = writeln!(file, "MT_{}={}", day.to_uppercase(), day_times_fmt.join(",")) {
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
       let _ = writeln!(file, "MT_FILE=\"{}\"", task.file.display())?;

       // write slide delay

       let _ = writeln!(file, "MT_SLIDE_DELAY=\"{}\"", task.slide_delay)?;

      } else {
       eprintln!("Could not find home directory.");
       IoError::other("Could not find home directory");
   }
   Ok(())
}

/// Before the program starts, it unmounts and remounts any usb drives.
/// This is  called in order to unmount and remount any usbs using the naming conventions
/// that the mediatimer_init uses. The mount points for usb drives must be standardised in
/// order for this program to work. The program is designed this way so that it can be utilised 
/// by non-technical users.

fn main() -> Result<(), Box<dyn Error>> {

    // issue command to pause mediatimer_init
    // systemctl --user stop mediatimer_init.service
    let _stop_mediatimer_init = Command::new("systemctl")
        .arg("--user")
        .arg("stop")
        .arg("mediatimer_init.service")
        .output()
        .expect("Media Timer not restarted");


    // Find and load any existing config for the user
    // This is hard coded, as the user will always be named "fun"
    let username = whoami::username();
    let env_dir_path: PathBuf =["/home/", &username, ".mediatimer_config/vars"].iter().collect();


    // set up the config vars
    let mut file = PathBuf::new();
    let mut slide_delay: u32 = 5;
    let mut proc_type = ProcType::Video;
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
                "MT_PROCTYPE" => proc_type = match value.to_lowercase().as_str() {
                    "video" => ProcType::Video,
                    "audio" => ProcType::Audio,
                    "image" => ProcType::Image,
                    "slideshow" => ProcType::Slideshow,
                    "browser" => ProcType::Browser,
                    "executable" => ProcType::Executable,
                    &_ => ProcType::Video
                },
                "MT_AUTOLOOP" => auto_loop = match value.as_str() {
                    "true" => Autoloop::Yes,
                    "false" => Autoloop::No,
                    &_ => Autoloop::No
                },
                "MT_FILE" => file.push(value.as_str()),
                "MT_SLIDE_DELAY" => slide_delay = value.parse::<u32>().unwrap(),
                "MT_SCHEDULE" => schedule = match value.as_str() {
                    "true" => AdvancedSchedule::Yes,
                    "false" => AdvancedSchedule::No,
                    &_ => AdvancedSchedule::No
                },
                "MT_MONDAY" => monday = to_weekday(value, Weekday::Monday(Vec::new()))?,
                "MT_TUESDAY" => tuesday = to_weekday(value, Weekday::Tuesday(Vec::new()))?,
                "MT_WEDNESDAY" => wednesday = to_weekday(value, Weekday::Wednesday(Vec::new()))?,
                "MT_THURSDAY" => thursday = to_weekday(value, Weekday::Thursday(Vec::new()))?,
                "MT_FRIDAY" => friday = to_weekday(value, Weekday::Friday(Vec::new()))?,
                "MT_SATURDAY" => saturday = to_weekday(value, Weekday::Saturday(Vec::new()))?,
                "MT_SUNDAY" => sunday = to_weekday(value, Weekday::Sunday(Vec::new()))?,
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
    // returns Ok(ProcType) e.g. Ok(ProcType::Video)
    
    let proctype = ProcTypeWidget::new(proc_type).run(&mut terminal)?;
    let can_be_dir: bool = match &proctype {
        ProcType::Slideshow => true,
        _ => false
    };

    // return Ok(FileSelectType)
    let file_path = FileSelectWidget::new(file, can_be_dir).run(&mut terminal)?;
    
    let advanced_schedule = AdvancedScheduleWidget::new(schedule).run(&mut terminal)?;

    if advanced_schedule == AdvancedSchedule::Yes {
        //returns Ok(Timings)
        timings = TimingsWidget::new(timings).run(&mut terminal)?;
    }
    
    let is_media_type: bool = match &proctype {
        ProcType::Video => true,
        ProcType::Audio => true,
        _ => false
    };
    // return Ok(Autoloop) e.g. Ok(Autoloop::No)
    if is_media_type && advanced_schedule == AdvancedSchedule::Yes {
        auto_loop = AutoloopWidget::new(auto_loop).run(&mut terminal)?;
    }
    let task = Task::new(proctype, auto_loop, advanced_schedule, timings, file_path, slide_delay);

    // write_task 
    if let Err(e) = write_task(task) {
        eprintln!("Error writing tasks to env file: {}", e);
    }
    
    // loading issues the command to enable the mediatimer_init service
    let _loading = LoadingWidget::default().run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::io::{BufRead, BufReader};
    use tempfile::tempdir;
    use std::env;

    #[test]
    fn check_to_weekday() {
        let weekday = to_weekday(String::new(), Weekday::Monday(Vec::new())).unwrap();
        assert_eq!(weekday, Weekday::Monday(Vec::new()));
        
        let weekday = to_weekday(String::from("10:00:00-11:00:00"), Weekday::Tuesday(Vec::new())).unwrap();
        let schedule = vec!((String::from("10:00:00"), String::from("11:00:00")));
        assert_eq!(weekday, Weekday::Tuesday(schedule));
        

        let weekday = to_weekday(String::from("10:00:00-11:00:00,11:00:01-12:12:12"), Weekday::Wednesday(Vec::new())).unwrap();
        let schedule = vec!(
            (String::from("10:00:00"), String::from("11:00:00")),
            (String::from("11:00:01"), String::from("12:12:12"))
            );
        assert_eq!(weekday, Weekday::Wednesday(schedule));

        let weekday = to_weekday(String::from(" 10:00:00-11:00:00 , 11:00:01-12:12:12"), Weekday::Thursday(Vec::new())).unwrap();
        let schedule = vec!(
            (String::from("10:00:00"), String::from("11:00:00")),
            (String::from("11:00:01"), String::from("12:12:12"))
            );
        assert_eq!(weekday, Weekday::Thursday(schedule));
    }


     #[test]
    fn test_write_task() {
        // Create a temporary directory to use as home directory for the test
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        // Override the home directory for the test
        // Using an environment variable to mock home::home_dir()
        temp_dir.path().to_str().map(|home_path| {
            env::set_var("HOME", home_path);
        });

        // Create test data
        let proc_type = ProcType::Video;
        let auto_loop = Autoloop::Yes;
        let advanced_schedule = AdvancedSchedule::Yes;

        // Create schedule for Monday (10:00-11:00)
        let monday_schedule = vec![("10:00".to_string(), "11:00".to_string())];
        let monday = Weekday::Monday(monday_schedule);

        // Create schedule for Friday (15:30-16:45, 18:00-19:30)
        let friday_schedule = vec![
            ("15:30".to_string(), "16:45".to_string()),
            ("18:00".to_string(), "19:30".to_string())
        ];
        let friday = Weekday::Friday(friday_schedule);

        // Create empty schedules for other days
        let tuesday = Weekday::Tuesday(Vec::new());
        let wednesday = Weekday::Wednesday(Vec::new());
        let thursday = Weekday::Thursday(Vec::new());
        let saturday = Weekday::Saturday(Vec::new());
        let sunday = Weekday::Sunday(Vec::new());

        // Combine all schedules
        let timings = vec![monday, tuesday, wednesday, thursday, friday, saturday, sunday];

        // Test file path
        let file_path = PathBuf::from("/path/to/test/file.mp4");

        // Create task
        let task = Task::new(proc_type, auto_loop, advanced_schedule, timings, file_path, 7);

        // Write task to file
        write_task(task).expect("Failed to write task");

        // Check if config directory was created
        let config_dir = temp_path.join(".mediatimer_config");
        assert!(config_dir.exists(), "Config directory was not created");

        // Check if vars file was created
        let vars_file = config_dir.join("vars");
        assert!(vars_file.exists(), "Vars file was not created");

        // Read the contents of the vars file
        let file = fs::File::open(&vars_file).expect("Failed to open vars file");
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

        // Verify file contents
        assert!(lines.contains(&"MT_PROCTYPE=\"video\"".to_string()), "Missing or incorrect process type");
        assert!(lines.contains(&"MT_AUTOLOOP=\"true\"".to_string()), "Missing or incorrect autoloop setting");
        assert!(lines.contains(&"MT_SCHEDULE=\"true\"".to_string()), "Missing or incorrect schedule setting");
        assert!(lines.contains(&"MT_MONDAY=10:00-11:00".to_string()), "Missing or incorrect Monday schedule");
        assert!(lines.contains(&"MT_FRIDAY=15:30-16:45,18:00-19:30".to_string()), "Missing or incorrect Friday schedule");
        assert!(lines.contains(&"MT_FILE=\"/path/to/test/file.mp4\"".to_string()), "Missing or incorrect file path");

        // Verify empty schedules
        assert!(lines.contains(&"MT_TUESDAY=".to_string()), "Missing Tuesday schedule");
        assert!(lines.contains(&"MT_WEDNESDAY=".to_string()), "Missing Wednesday schedule");
        assert!(lines.contains(&"MT_THURSDAY=".to_string()), "Missing Thursday schedule");
        assert!(lines.contains(&"MT_SATURDAY=".to_string()), "Missing Saturday schedule");
        assert!(lines.contains(&"MT_SUNDAY=".to_string()), "Missing Sunday schedule");
    }

    #[test]
    fn test_write_task_no_advanced_schedule() {
        // Create a temporary directory to use as home directory for the test
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        // Override the home directory for the test
        temp_dir.path().to_str().map(|home_path| {
            env::set_var("HOME", home_path);
        });

        // Create test data with no advanced schedule
        let proc_type = ProcType::Browser;
        let auto_loop = Autoloop::No;
        let advanced_schedule = AdvancedSchedule::No;
        let timings = default_timings(); // Use default empty timings
        let file_path = PathBuf::from("/path/to/browser/page.html");

        // Create task
        let task = Task::new(proc_type, auto_loop, advanced_schedule, timings, file_path, 7);

        // Write task to file
        write_task(task).expect("Failed to write task");

        // Check if vars file was created
        let vars_file = temp_path.join(".mediatimer_config").join("vars");
        assert!(vars_file.exists(), "Vars file was not created");

        // Read the contents of the vars file
        let file = fs::File::open(&vars_file).expect("Failed to open vars file");
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

        // Verify file contents
        assert!(lines.contains(&"MT_PROCTYPE=\"browser\"".to_string()), "Missing or incorrect process type");
        assert!(lines.contains(&"MT_AUTOLOOP=\"false\"".to_string()), "Missing or incorrect autoloop setting");
        assert!(lines.contains(&"MT_SCHEDULE=\"false\"".to_string()), "Missing or incorrect schedule setting");
        assert!(lines.contains(&"MT_FILE=\"/path/to/browser/page.html\"".to_string()), "Missing or incorrect file path");

        // Verify empty schedules are still written
        for day in ["MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY", "SATURDAY", "SUNDAY"] {
            assert!(lines.contains(&format!("MT_{}=", day)), "Missing {} schedule", day);
        }
    }
}
