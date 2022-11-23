use std::cell::Cell;
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::rc::Rc;
use std::sync;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{self, sleep};

use anyhow::{anyhow, ensure, Result};
use gol::{Mask, Point};
use pancurses::{curs_set, endwin, init_pair, start_color, Input, COLOR_BLACK};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use std::{io, time::Duration};

mod gol;

type Board = gol::Board;

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
fn mk_pool(threads: usize) -> Result<rayon::ThreadPool> {
    Ok(rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?)
}

fn run_turn(board: Board, threads: u32) -> Result<Board> {
    let chunk_size = ((board.height() as f32) / threads as f32).ceil() as usize;
    let mut outboard = board.clone();
    outboard
        .pixels_mut()
        .par_chunks_mut(chunk_size)
        .for_each(|cs| {
            for (pt, v) in cs {
                //dbg!(*c.1, calc_px(&board, &c.0));
                **v = calc_px(&board, &pt);
            }
        });
    Ok(outboard)
}

fn read_pgm(f: &mut dyn Read) -> Result<Board> {
    let buf = BufReader::new(f);
    let bytes: Vec<u8> = buf
        .bytes()
        .map(|m| m.map_err(|e| anyhow!(e)))
        .collect::<Result<_>>()?;
    let (sections, pixels): (Vec<_>, Vec<_>) = bytes
        .split(|bv| {
            let b = *bv as char;
            b == ' ' || b == '\n' || b == '\r' || b == '\t'
        })
        .enumerate()
        .partition(|(i, _)| *i < 4);
    let sections: Vec<String> = sections
        .into_iter()
        .map(|(_, cs)| -> Result<String> {
            Ok(String::from_utf8(cs.iter().map(|c| *c as u8).collect())?)
        })
        .collect::<Result<_>>()?;
    let pixels: Vec<_> = pixels.into_iter().flat_map(|m| m.1).collect();
    ensure!(sections.len() == 4, "invalid number of sections");
    ensure!(sections[0] == "P5", "not a P5 pgm");
    let width: u32 = sections[1].parse()?;
    let maxgrey: u16 = sections[3].parse()?;
    ensure!(maxgrey < 256, "max grey too high!");
    Ok(Board::new(
        width,
        pixels
            .into_iter()
            .map(|p| *p as u8 == maxgrey as u8)
            .collect(),
    ))
}
fn write_pgm(b: &Board, f: &mut dyn Write) -> Result<()> {
    f.write_all(&format!("P5\n{} {} 255\n", b.width(), b.height()).as_bytes())?;
    let px = b
        .pixels()
        .into_iter()
        .map(|p| if p.1 { 255 } else { 0 })
        .collect::<Vec<_>>();
    f.write_all(&px)?;
    Ok(())
}
enum Event {
    TurnEnd(Board),
    KeyPress(Input),
}

struct SessionWin {
    win: pancurses::Window,
}
impl Deref for SessionWin {
    type Target = pancurses::Window;

    fn deref(&self) -> &Self::Target {
        return &self.win;
    }
}
impl Drop for SessionWin {
    fn drop(&mut self) {
        endwin();
    }
}
impl SessionWin {
    fn initscr() -> Self {
        let win = pancurses::initscr();
        start_color();
        Self { win }
    }
}

#[derive(Copy, Clone)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

enum CellColours {
    Dead = 0,
    Alive = 1,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let threads = args.get(2).and_then(|m| m.parse().ok()).unwrap_or(4);
    let mut infile = std::fs::File::open(args.get(1).expect("no input filename?"))
        .expect("failed to open input file");
    let initial = read_pgm(&mut infile)?;
    let mut turn = 0;
    let (sx, tx) = std::sync::mpsc::channel();
    let bsx = sx.clone();
    let running = AtomicBool::new(true);
    thread::scope(|s| {
        let mut curr = initial.clone();
        let running = &running;
        s.spawn(move || -> Result<()> {
            mk_pool(threads as usize)
                .expect("failed to create threadpool")
                .install(move || -> Result<()> {
                    while running.load(sync::atomic::Ordering::SeqCst) {
                        curr = run_turn(curr, threads).expect("failed to run turn");
                        let r = bsx.send(Event::TurnEnd(curr.clone()));
                        if r.is_err() {
                            break;
                        }
                    }
                    Ok(())
                })?;
            Ok(())
        });

        let win = SessionWin::initscr();
        win.clear();
        win.refresh();
        curs_set(0);
        let ksx = sx.clone();
        s.spawn(move || {
            let w = pancurses::newwin(0, 0, 0, 0);
            w.nodelay(true);
            w.keypad(true);
            while running.load(sync::atomic::Ordering::SeqCst) {
                match w.getch() {
                    None => {
                        sleep(Duration::from_millis(1));
                        continue;
                    }
                    Some(c) => {
                        let r = ksx.send(Event::KeyPress(c));
                        if r.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        start_color();
        init_pair(
            CellColours::Dead as i16,
            pancurses::COLOR_BLACK,
            pancurses::COLOR_BLACK,
        );
        init_pair(
            CellColours::Alive as i16,
            pancurses::COLOR_WHITE,
            pancurses::COLOR_WHITE,
        );
        init_pair(3, pancurses::COLOR_GREEN, pancurses::COLOR_BLACK);

        let mut offset = Point { x: 0, y: 0 };
        while running.load(sync::atomic::Ordering::SeqCst) {
            let ev = tx.recv().unwrap();
            match ev {
                Event::TurnEnd(b) => {
                    turn += 1;
                    offset.remap(b.width(), b.height());
                    win.color_set(CellColours::Dead as i16);
                    for py in 0..win.get_max_y() {
                        for px in 0..win.get_max_x() {
                            let pt = Point {
                                x: px as i64 + offset.x,
                                y: py as i64 + offset.y,
                            };
                            let v = b[pt.clone()];

                            let c = if v {
                                CellColours::Alive
                            } else {
                                CellColours::Dead
                            };
                            win.color_set(c as i16);
                            win.mvaddch(py, px, ' ');
                        }
                    }
                    win.color_set(3);
                    win.mvaddstr(0, 0, format!("turn {}", turn));
                    win.refresh();
                }
                Event::KeyPress(Input::KeyLeft) | Event::KeyPress(Input::Character('h')) => {
                    offset.x -= 1
                }
                Event::KeyPress(Input::KeyRight) | Event::KeyPress(Input::Character('l')) => {
                    offset.x += 1
                }
                Event::KeyPress(Input::KeyUp) | Event::KeyPress(Input::Character('k')) => {
                    offset.y -= 1
                }
                Event::KeyPress(Input::KeyDown) | Event::KeyPress(Input::Character('j')) => {
                    offset.y += 1
                }
                Event::KeyPress(Input::KeyEIC) | Event::KeyPress(Input::Character('q')) => {
                    running.store(false, sync::atomic::Ordering::SeqCst);
                }
                _ => (),
            }
        }
    });
    endwin();
    Ok(())
}
