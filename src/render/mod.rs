use crate::gol::Board;

pub mod cursive;

pub trait Renderer {
    fn running(&self) -> bool;
    fn tick(&mut self);
    fn render_board(self, b: &Board) -> Self;
}
