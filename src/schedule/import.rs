use crate::{
    Weekday,
    to_weekday

};
use std::{
    env,
    path::{
        PathBuf,
    },
};

use crate::Timings;

pub fn import_schedule(schedule_path: PathBuf) -> Vec<Weekday>  {
    // set up the config vars
    let timings: Timings = Vec::with_capacity(7);
    let mut monday: Weekday = Weekday::Monday(Vec::with_capacity(2));
    let mut tuesday: Weekday = Weekday::Tuesday(Vec::with_capacity(2));
    let mut wednesday: Weekday = Weekday::Wednesday(Vec::with_capacity(2));
    let mut thursday: Weekday = Weekday::Thursday(Vec::with_capacity(2));
    let mut friday: Weekday = Weekday::Friday(Vec::with_capacity(2));
    let mut saturday: Weekday = Weekday::Saturday(Vec::with_capacity(2));
    let mut sunday: Weekday = Weekday::Sunday(Vec::with_capacity(2));
    
    if schedule_path.exists() {
        if let Err(_) = dotenvy::from_path_override(schedule_path.as_path()) {
            eprintln!("Cannot find env vars at path: {}", schedule_path.display());
        }
        // parse the environmental vars
        for (key, value) in env::vars() {
            match key.as_str() {
                // TODO re-implement result return values
                "MT_MONDAY" => monday = to_weekday(value, Weekday::Monday(Vec::new())).expect("weekday not parsed"),
                "MT_TUESDAY" => tuesday = to_weekday(value, Weekday::Tuesday(Vec::new())).expect("weekday not parsed"),
                "MT_WEDNESDAY" => wednesday = to_weekday(value, Weekday::Wednesday(Vec::new())).expect("weekday not parsed"),
                "MT_THURSDAY" => thursday = to_weekday(value, Weekday::Thursday(Vec::new())).expect("weekday not parsed"),
                "MT_FRIDAY" => friday = to_weekday(value, Weekday::Friday(Vec::new())).expect("weekday not parsed"),
                "MT_SATURDAY" => saturday = to_weekday(value, Weekday::Saturday(Vec::new())).expect("weekday not parsed"),
                "MT_SUNDAY" => sunday = to_weekday(value, Weekday::Sunday(Vec::new())).expect("weekday not parsed"),
                _ => {}
            }
        }

        return vec![monday, tuesday, wednesday, thursday, friday, saturday, sunday];
    }

    timings
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn check_import_schedule() {
        // create file at temporary path
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let mut temp_path = temp_dir.path().to_path_buf();
        temp_path.push("schedule.mt");

        // write schedule to file
        // Create schedule for Monday (10:00-11:00)
        let monday_schedule = vec![("10:00:00".to_string(), "11:00:00".to_string())];
        let monday = Weekday::Monday(monday_schedule);

        // Create schedule for Friday (15:30-16:45, 18:00-19:30)
        let friday_schedule = vec![
            ("15:30:00".to_string(), "16:45:00".to_string()),
            ("18:00".to_string(), "19:30:00".to_string())
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

        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path).expect("file could not be opened");
    
        // This function should be converted to a closure
       fn format_print_day_schedule(day: String, schedule: Schedule, mut file: fs::File) {
           let day_times_fmt: Vec<String> = schedule.iter().map(|i| format!("{}-{}", i.0, i.1)).collect();
           if let Err(e) = writeln!(file, "MT_{}={}", day.to_uppercase(), day_times_fmt.join(",")) {
               eprintln!("Could not write to file: {}", e);
           }
       }
       for timing in timings.iter() {
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
 
        let imported_schedule = import_schedule(temp_path);

        assert_eq!(imported_schedule[0], Weekday::Monday(vec!(
                    (
                        String::from("10:00:00"),
                        String::from("11:00:00")
                    )
        )));
        
    }
}



