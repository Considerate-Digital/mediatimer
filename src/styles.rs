use ratatui::{
    style::{
        palette::tailwind::{SLATE, WHITE},
        Color, Modifier, Style, 
    },
};

const PINK: Color = Color::Rgb(255, 214, 228);
const PINK_LIGHT: Color = Color::Rgb(255, 219, 230);
const PINK_SELECTED: Color = Color::Rgb(255, 229, 237);
const BLUE: Color = Color::Rgb(15, 40, 48);
const BLUE_ALT: Color = Color::Rgb(32, 61, 71);
const GREEN: Color = Color::Rgb(4, 211, 126);

pub const ITEM_HEADER_STYLE: Style = Style::new().fg(WHITE).bg(BLUE);
pub const NORMAL_ROW_BG: Color = PINK;
pub const ALT_ROW_BG_COLOR: Color = PINK_LIGHT;
pub const SELECTED_STYLE: Style = Style::new().bg(PINK_SELECTED).fg(BLUE).add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = BLUE;
pub const TEXT_DIR_COLOR: Color = BLUE;
pub const FOOTER_STYLE: Style = Style::new().bg(BLUE).fg(WHITE);

