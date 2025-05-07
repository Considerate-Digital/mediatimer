use ratatui::{
    style::{
        palette::tailwind::{SLATE, WHITE},
        Color, Modifier, Style, 
    },
};

// #ffd6ff
const PINK: Color = Color::Rgb(255, 214, 255);
// #ffdcff
const PINK_LIGHT: Color = Color::Rgb(255, 220, 255);
// #ffe6ff
const PINK_SELECTED: Color = Color::Rgb(255, 230, 255);
// #0f2830
const BLUE: Color = Color::Rgb(15, 40, 48);
// #12303a
const BLUE_LIGHT: Color = Color::Rgb(18, 48, 58);
// #173742
const BLUE_LIGHTEST: Color = Color::Rgb(23, 55, 66);
const BLUE_ALT: Color = Color::Rgb(32, 61, 71);
const GREEN: Color = Color::Rgb(4, 211, 126);

pub const ITEM_HEADER_STYLE: Style = Style::new().fg(WHITE).bg(BLUE);
pub const NORMAL_ROW_BG: Color = PINK;
pub const ALT_ROW_BG_COLOR: Color = PINK_LIGHT;
pub const SELECTED_STYLE: Style = Style::new().bg(BLUE).fg(WHITE).add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = BLUE;
pub const TEXT_DIR_COLOR: Color = BLUE;
pub const FOOTER_STYLE: Style = Style::new().bg(BLUE).fg(WHITE);

