use std::{
    fs,
    io,
    io::Write,
    io::Error as IoError,
    thread,
    time::Duration,
    path::{
        PathBuf,
    },
    process::Command
};
use regex::Regex;
use std::error::Error;
use ratatui::{
    prelude::CrosstermBackend,
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
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

mod landing;
use crate::landing::LandingWidget;

mod reboot;
use crate::reboot::RebootWidget;

/*
mod timings;
use crate::timings::TimingsWidget;
*/

const ITEM_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const TEXT_DIR_COLOR: Color = GREEN.c200;


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
#[derive(Debug, Display, PartialEq)]
enum Reboot {
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
    /*
    fn set_loop(&mut self, auto_loop: Autoloop) {
        self.auto_loop = auto_loop;
    }
    fn set_proc_type(&mut self, p_type: ProcType) {
        self.proc_type = p_type;
    }
    fn set_weekday(&mut self, wd: Weekday) {
        self.timings.push(wd);
    }
    */
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
       let _i = writeln!(file, "ML_PROCTYPE=\"{}\"", task.proc_type.to_string().to_lowercase())?;

       //write autoloop
        let _i = writeln!(file, "ML_AUTOLOOP=\"{}\"", match task.auto_loop {
            Autoloop::Yes => "true",
            Autoloop::No => "false"
        });

       // TODO write timings
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
       writeln!(file, "ML_FILE=\"{}\"", task.file.display())?;

       // advanced use
       writeln!(file, "# Change this to 'true' if you want to use a custom schedule");
       writeln!(file, "ML_SCHEDULE=\"false\"");

       //full schedule layout
       let schedule = "#ML_MONDAY=\"09:00-12:00,13:00-17:00\"\n#ML_TUESDAY=\"09:00-12:00,13:00-17:00\"\n#ML_WEDNESDAY=\"09:00-12:00,13:00-17:00\"\n#ML_THURSDAY=\"09:00-12:00,13:00-17:00\"\n#ML_FRIDAY=\"09:00-12:00,13:00-17:00\"\n#ML_SATURDAY=\"09:00-12:00,13:00-17:00\"\n#ML_SUNDAY=\"09:00-12:00,13:00-17:00\"\n";
       writeln!(file, "# Remove the '#' at the start of each day that you require a customised schedule for.\n# Edit the timings and add new entries if needed.\n# Make sure the timings have the format START-END and are comma (',') separated with no spaces.\n# Schedule timings can be specified in either minute-format (10:00) or second-format (10:00:00)\n# Note that the auto-loop feature only applies to media files and you must implement internal loops yourself for browser-based or executable files.");
       writeln!(file, "{}", schedule);
            

   } else {
       eprintln!("Could not find home directory.");
       IoError::other("Could not find home directory");
   }
   Ok(())
}

enum Usb {
    SDA1,
    SDA2,
    SDA3,
    SDA4,
    SDB1,
    SDB2,
    SDB3,
    SDB4,
    SDC1,
    SDC2,
    SDC3,
    SDC4,
    UNKNOWN
}

impl Usb {
    fn as_str(&self) -> &'static str {
        match self {
            Usb::SDA1 => "sda1", 
            Usb::SDA2 => "sda2", 
            Usb::SDA3 => "sda3", 
            Usb::SDA4 => "sda4",
            Usb::SDB1 => "sdb1", 
            Usb::SDB2 => "sdb2", 
            Usb::SDB3 => "sdb3",
            Usb::SDB4 => "sdb4",
            Usb::SDC1 => "sdc1",
            Usb::SDC2 => "sdc2",
            Usb::SDC3 => "sdc3",
            Usb::SDC4 => "sdc4",
            Usb::UNKNOWN => ""
        }
    }
}


fn find_mount_drives() -> Result<(), Box<dyn Error>> {
    println!("Finding and mounting drives");
    // check with usbs are available 
    let all_drives = Command::new("lsblk")
        .arg("-l")
        .arg("-o")
        .arg("NAME,HOTPLUG")
        .output()
        .expect("some drives");
    
    let all_drives_string = String::from_utf8_lossy(&all_drives.stdout);
    
    for line in all_drives_string.lines() {
        let re = Regex::new(r"sd[a,b,c][1-4]").unwrap();
        if re.is_match(line) {
            let drive_info = line.split(' ')
                .filter(|d| *d != "" )
                .collect::<Vec<_>>();
                if drive_info[1] == "1" { 
                    
                    // have the thread sleep for one second as puppy umount sometimes fails
                    let one_second = Duration::from_millis(1000); 
                    thread::sleep(one_second);
                    // unmount the drive before going further
                    let unmount_com = Command::new("umount")
                        .arg("/dev/".to_owned() + drive_info[0])
                        .output()
                        .expect("Failed to unmount usb drive");

                    println!("{:?}", unmount_com);
                    let drive = match drive_info[0] {
                        "sda1" => Usb::SDA1,
                        "sda2" => Usb::SDA2,
                        "sda3" => Usb::SDA3,
                        "sda4" => Usb::SDA4,
                        "sdb1" => Usb::SDB1,
                        "sdb2" => Usb::SDB2,
                        "sdb3" => Usb::SDB3,
                        "sdb4" => Usb::SDB4,
                        "sdc1" => Usb::SDC1,
                        "sdc2" => Usb::SDC2,
                        "sdc3" => Usb::SDC3,
                        "sdc4" => Usb::SDC4,
                        &_ => Usb::UNKNOWN

                    };
                    mount_usb(drive)?;
                }
        }
    }
    Ok(())
}

fn mount_usb(drive: Usb) -> Result<(), Box<dyn Error>> {

    let mnt_dir: String = match drive {
        Usb::SDA1 => format!("usb_{}", Usb::SDA1.as_str()),
        Usb::SDA2 => format!("usb_{}", Usb::SDA2.as_str()),
        Usb::SDA3 => format!("usb_{}", Usb::SDA3.as_str()),
        Usb::SDA4 => format!("usb_{}", Usb::SDA4.as_str()),
        Usb::SDB1 => format!("usb_{}", Usb::SDB1.as_str()),
        Usb::SDB2 => format!("usb_{}", Usb::SDB2.as_str()),
        Usb::SDB3 => format!("usb_{}", Usb::SDB3.as_str()),
        Usb::SDB4 => format!("usb_{}", Usb::SDB4.as_str()),
        Usb::SDC1 => format!("usb_{}", Usb::SDC1.as_str()),
        Usb::SDC2 => format!("usb_{}", Usb::SDC2.as_str()),
        Usb::SDC3 => format!("usb_{}", Usb::SDC3.as_str()),
        Usb::SDC4 => format!("usb_{}", Usb::SDC4.as_str()),
        Usb::UNKNOWN => "".to_string()

    };
    let drive_name = match drive {
        Usb::SDA1 => Usb::SDA1.as_str(), 
        Usb::SDA2 => Usb::SDA2.as_str(), 
        Usb::SDA3 => Usb::SDA3.as_str(), 
        Usb::SDA4 => Usb::SDA4.as_str(),
        Usb::SDB1 => Usb::SDB1.as_str(), 
        Usb::SDB2 => Usb::SDB2.as_str(), 
        Usb::SDB3 => Usb::SDB3.as_str(),
        Usb::SDB4 => Usb::SDB4.as_str(),
        Usb::SDC1 => Usb::SDC1.as_str(),
        Usb::SDC2 => Usb::SDC2.as_str(),
        Usb::SDC3 => Usb::SDC3.as_str(),
        Usb::SDC4 => Usb::SDC4.as_str(),
        Usb::UNKNOWN => ""
    };
    let _mount_drive = Command::new("mount")
            .arg("/dev/".to_owned() + drive_name)
            // tell mount to make the target dir
            .arg("-o")
            .arg("rw,x-mount.mkdir")
            .arg("/mnt/".to_owned() + &mnt_dir)
            .output()
            .expect("failed to mount");

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

    /// Before the program starts, it unmounts and remounts any usb drives.
    /// This is  called in order to unmount and remount any usbs using the naming conventions
    /// that the medialoop_init uses. The mount points for usb drives must be standardised in
    /// order for this program to work.
    let _usb_drive_mount = find_mount_drives()?;


    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
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


    //returns Ok(Timings)
    //let timings = TimingsWidget::default().run(&mut terminal)?;


    // if the selected file is on a usb stick
    // edit fstab to automount that usb
    let timings = Vec::new();
    let task = Task::new(proctype, autoloop, timings, file_path);

    // write a function that writes the task to a specific env file
    // write_task 
    if let Err(e) = write_task(task) {
        eprintln!("Error writing tasks to env file: {}", e);
    }


    // return Ok(Reboot) e.g. Ok(Reboot::No)
    let mut reboot = RebootWidget::default().run(&mut terminal)?;

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
                println!("rebooting");
                /*
                let _reboot = Command::new("reboot")
                    .output()
                    .expect("could not reboot");
                */
            }
            Reboot::No => {}
        }

    Ok(())
}
