

use crate::{
    Weekday,
    Schedule,
    to_weekday

};
use std::{
    env,
    fs,
    io,
    io::Write,
    io::Error as IoError,
    path::{
        PathBuf,
    },
};

use crate::Timings;

// accepts one arg: a full week schedule as TimingCollection vec<TimingsEntry>
// result return type temporarily removed
pub fn export_schedule(timings: Vec<Weekday>) {
    // assembles schedule and writes it to file
   if let Some(dir) = home::home_dir() {
        // check if dir exists
        let mut dir_path = PathBuf::from(dir);
        // check if the mediatimer directory exists in home
        if dir_path.as_path().is_dir() == false {
            // create the mediatimer directory if it does not exist
            if let Err(er) = fs::create_dir(dir_path.as_path()) {
               eprintln!("Directory could not be created: {}", er);
               IoError::other("Could not create mediatimer directory.");
            }
        }

        dir_path.push("schedule.mt");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&dir_path);
    
       // This function should be converted to a closure
       fn format_print_day_schedule(day: String, schedule: Schedule, mut file: fs::File) {
           let day_times_fmt: Vec<String> = schedule.iter().map(|i| format!("{}-{}", i.0, i.1)).collect();
           if let Err(e) = writeln!(file, "MT_{}={}", day.to_uppercase(), day_times_fmt.join(",")) {
               eprintln!("Could not write to file: {}", e);
           }
       }
       for timing in timings.iter() {
           if let Ok(file_clone) = file.as_ref().expect("file does not exist").try_clone() {
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
       
      } else {
       eprintln!("Could not find home directory.");
       IoError::other("Could not find home directory");
   }
}
