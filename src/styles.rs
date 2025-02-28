use ratatui::{
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE, WHITE},
        Color, Modifier, Style, 
    },
};

pub const ITEM_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
pub const NORMAL_ROW_BG: Color = SLATE.c950;
pub const ALT_ROW_BG_COLOR: Color = SLATE.c900;
pub const SELECTED_STYLE: Style = Style::new().bg(SLATE.c600).add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = SLATE.c200;
pub const TEXT_DIR_COLOR: Color = GREEN.c200;
pub const FOOTER_STYLE: Style = Style::new().bg(SLATE.c700).fg(WHITE);

