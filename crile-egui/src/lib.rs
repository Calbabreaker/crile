mod egui_context;
mod inspect;

pub use egui_context::*;
pub use inspect::*;

pub fn to_egui_pos(vec: glam::Vec2) -> egui::Pos2 {
    egui::pos2(vec.x, vec.y)
}

pub fn from_egui_vec(vec: egui::Vec2) -> glam::Vec2 {
    glam::vec2(vec.x, vec.y)
}
