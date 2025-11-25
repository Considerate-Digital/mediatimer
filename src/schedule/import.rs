

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

pub fn import_schedule(schedule_path: PathBuf) -> Vec<Weekday>  {
    let username = whoami::username();
    // set up the config vars
    let mut timings: Timings = Vec::with_capacity(7);
    let mut monday: Weekday = Weekday::Monday(Vec::with_capacity(2));
    let mut tuesday: Weekday = Weekday::Tuesday(Vec::with_capacity(2));
    let mut wednesday: Weekday = Weekday::Wednesday(Vec::with_capacity(2));
    let mut thursday: Weekday = Weekday::Thursday(Vec::with_capacity(2));
    let mut friday: Weekday = Weekday::Friday(Vec::with_capacity(2));
    let mut saturday: Weekday = Weekday::Saturday(Vec::with_capacity(2));
    let mut sunday: Weekday = Weekday::Sunday(Vec::with_capacity(2));
    
    if schedule_path.exists() {
        if let Err(e) = dotenvy::from_path_override(schedule_path.as_path()) {
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



