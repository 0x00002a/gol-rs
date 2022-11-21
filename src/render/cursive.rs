use std::rc::Rc;

use cursive::{theme::Color, view::Nameable, Cursive, CursiveExt, CursiveRunner, Printer, View};

use crate::gol::{Board, Point};

use super::Renderer;

pub struct CursiveRender {
    c: cursive::CursiveRunnable,
}

impl CursiveRender {
    pub fn new(b: &Board) -> Self {
        let view = BoardView { b: b.clone() };
        let mut c = cursive::default();
        c.add_layer(view.with_name("board_view"));
        c.add_global_callback('q', |s| s.quit());
        Self { c }
    }
}
unsafe impl Sync for CursiveRender {}

struct BoardView {
    b: Board,
}

impl View for BoardView {
    fn draw(&self, printer: &Printer) {
        /*self.b.pixels().into_iter().for_each(|(pt, v)| {
            let c = if v {
                Color::Rgb(0, 0, 0)
            } else {
                Color::Rgb(255, 255, 255)
            };
            printer.with_color(c.into(), |p| {
                p.print((pt.x as i32, pt.y as i32), " ");
            });
        });*/
    }
}

impl Renderer for CursiveRender {
    fn render_board(mut self, b: &Board) -> Self {
        self.c.find_name::<BoardView>("board_view").unwrap().b = b.clone();
        self
    }
    fn running(&self) -> bool {
        self.c.is_running()
    }

    fn tick(&mut self) {
        let mut r = self.c.runner();
        //r.refresh();
        /*r.step();
        r.is_running()*/
        r.step();
    }
}
