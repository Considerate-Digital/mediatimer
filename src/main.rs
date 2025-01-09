use clokwerk::{Scheduler, TimeUnits, Job};
// Import weekdays and weekday
use clokwerk::Interval::*;
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::env;
use std::error::Error;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
    Frame,
};
use crossterm::event::{self, Event};

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World");
    frame.render_widget(text, frame.area());
}

fn main() -> Result<(), Box<dyn Error>> {
    // use this dir .env for testing
    dotenvy::from_path(Path::new("/home/alex/medialoop/src/.env"))?;

    for (key, value) in env::vars() {
        match key.as_str() {
            "MLWEEKDAYS" => println!("{}", value),
            "MLSTART" => println!("{}", value),
            "MLEND" => println!("{}", value),
            _ => {}
        }
    }

    let mut terminal = ratatui::init();
    
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }

    }
    ratatui::restore();



    Ok(())
}
