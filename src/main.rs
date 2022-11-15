use gol::{Mask, Point};

mod gol;

type Board = gol::Board<bool>;

fn calc_px(board: &Board, pt: &Point) -> bool {
    let alive = board.neighbors(pt).iter().filter(|p| **p).count();
    let imalive = board[pt.clone()];
    if !imalive && alive == 3 {
        return true;
    } else if imalive && (alive < 2 || alive > 3) {
        return false;
    } else {
        return imalive;
    }
}

fn run_turn(board: &mut Board) {
    for y in 0..board.width() {
        for x in 0..board.height() {
            let pt = Point {
                x: x as i64,
                y: y as i64,
            };
            board[pt] = calc_px(&board, &pt);
        }
    }
}

fn main() {
    println!("Hello, world!");
}
