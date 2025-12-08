use ratatui::{
    layout::{
        Rect
    }
};

pub fn popup_area(area: Rect) -> Rect {
    Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        }
}
