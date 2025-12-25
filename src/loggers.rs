use log::{LevelFilter};
use std::error::Error;

//use std::io::Error;
use systemd_journal_logger::JournalLog;

pub fn setup_logger() -> Result<(), Box<dyn Error>> {
    JournalLog::new()?.install()?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

#[macro_export] 
macro_rules! logi {
    ($($t:tt)*) => {{
       info!($($t)*); 
       println!($($t)*);
    }};
}

#[macro_export] 
macro_rules! logw {
    ($($t:tt)*) => {{
       warn!($($t)*); 
       println!($($t)*);
    }};
}

#[macro_export] 
macro_rules! loge {
    ($($t:tt)*) => {{
       error!($($t)*); 
       eprintln!($($t)*);
    }};
}

pub fn log_info(message: &str) {
    /*
   info!(message); 
   println!(message);
    */
}

#[allow(dead_code)]
pub fn log_warn(message: &str) {
    /*
    warn!(message);
    println!(message);
    */
}

pub fn log_error(message: &str) {
    /*
    error!(message);
    eprintln!(message);
    */
}

