

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::{BufRead, BufReader};

    #[test]
    fn check_export_schedule() {
        // override the home directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        // Override the home directory for the test
        // Using an environment variable to mock home::home_dir()
        temp_dir.path().to_str().map(|home_path| {
            env::set_var("HOME", home_path);
        });

        // create a set of timings
        // Create schedule for Monday (10:00-11:00)
        let monday_schedule = vec![("10:00:00".to_string(), "11:00:00".to_string())];
        let monday = Weekday::Monday(monday_schedule);

        // Create schedule for Friday (15:30-16:45, 18:00-19:30)
        let friday_schedule = vec![
            ("15:30:00".to_string(), "16:45:00".to_string()),
            ("18:00:00".to_string(), "19:30:00".to_string())
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
    
        // add the schedule.mt to the temporary home dir path
        let schedule_path = temp_path.join("schedule.mt");
        println!("{}", schedule_path.display());
    
        // run the export function
        let _export_schedule = export_schedule(timings); 


        assert!(schedule_path.exists(), "schedule.mt was not created");

        let mut file = fs::File::open(&schedule_path).expect("file could not be opened");

        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
        
        assert!(lines.contains(&"MT_MONDAY=10:00:00-11:00:00".to_string()), "Missing or incorrect Monday schedule");
        assert!(lines.contains(&"MT_FRIDAY=15:30:00-16:45:00,18:00:00-19:30:00".to_string()), "Missing or incorrect Friday schedule");

    }
}

