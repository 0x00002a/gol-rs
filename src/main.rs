use std::any::Any;
use std::io::{BufReader, Read};
use std::ops::Deref;
use std::panic::PanicInfo;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::{panic, sync};

use anyhow::{anyhow, ensure, Result};
use args::Args;
use bgrid::{Charset, Frame};
use clap::Parser;
use gol::{Mask, Point};
use pancurses::{curs_set, endwin, init_pair, noecho, start_color, Input};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use scopeguard::defer;
use std::time::Duration;

mod args;
mod bgrid;
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

fn check(r: i32) -> Result<()> {
    if r != 0 {
        Err(anyhow!("pancurses returned error code {}", r))
    } else {
        Ok(())
    }
}
fn run_event_loop(
    running: &AtomicBool,
    tx: Receiver<Event>,
    bg: char,
    chset: Charset,
) -> Result<()> {
    let win = SessionWin::initscr();
    win.keypad(true);
    win.nodelay(true);
    win.clear();
    win.refresh();
    curs_set(0);
    noecho();

    defer! { running.store(false, sync::atomic::Ordering::SeqCst); }
    start_color();
    init_pair(0, pancurses::COLOR_WHITE, pancurses::COLOR_BLACK);
    init_pair(3, pancurses::COLOR_GREEN, pancurses::COLOR_BLACK);

    let scroll_inc: i64 = (win.get_max_x().max(win.get_max_y()) / 10).into();
    let mut offset = Point { x: 0, y: 0 };
    let mut turn = 0;
    while running.load(sync::atomic::Ordering::SeqCst) {
        let mut ev = Option::None;
        while let None = ev {
            ev = win
                .getch()
                .map(|c| Event::KeyPress(c))
                .or_else(|| tx.try_recv().ok());
            if ev.is_none() {
                sleep(Duration::from_millis(1));
            }
        }
        let ev = ev.unwrap();
        match ev {
            Event::TurnEnd(b) => {
                turn += 1;
                offset.remap(b.width(), b.height());
                let viewport = Mask {
                    x: (offset.x) as u32,
                    y: (offset.y) as u32,
                    w: (win.get_max_x()) as u32,
                    h: (win.get_max_y()) as u32,
                };
                let frame = Frame::new(b.clone(), viewport);
                let screen_view = Mask {
                    x: win.get_beg_x() as u32,
                    y: win.get_beg_y() as u32,
                    w: win.get_max_x() as u32,
                    h: win.get_max_y() as u32,
                };
                win.color_set(0);
                win.clear();
                frame
                    .render(bg, chset)
                    .into_iter()
                    .map(|(pt, c)| {
                        if !screen_view.contains(&pt) {
                            Err(anyhow!(
                                "tried to draw outside the viewport: {} not in {}",
                                pt,
                                screen_view
                            ))
                        } else {
                            let r = check(win.mvaddstr(pt.y as i32, pt.x as i32, String::from(c)))
                                .map_err(|e| {
                                    anyhow!("check failed pt {} v: {} e: {}", pt, screen_view, e)
                                });
                            if r.is_ok()
                                || pt.x == (win.get_max_x() - 1) as i64
                                    && pt.y == (win.get_max_y() - 1) as i64
                            {
                                Ok(())
                            } else {
                                r
                            }
                        }
                    })
                    .collect::<Result<_>>()?;
                win.color_set(3);
                win.mvaddstr(0, 0, format!("turn  {}", turn));
                win.mvaddstr(1, 0, format!("alive {}", b.alive()));
                win.refresh();
            }
            Event::KeyPress(Input::KeyLeft) | Event::KeyPress(Input::Character('h')) => {
                offset.x -= scroll_inc
            }
            Event::KeyPress(Input::KeyRight) | Event::KeyPress(Input::Character('l')) => {
                offset.x += scroll_inc
            }
            Event::KeyPress(Input::KeyUp) | Event::KeyPress(Input::Character('k')) => {
                offset.y -= scroll_inc
            }
            Event::KeyPress(Input::KeyDown) | Event::KeyPress(Input::Character('j')) => {
                offset.y += scroll_inc
            }
            Event::KeyPress(Input::KeyEIC) | Event::KeyPress(Input::Character('q')) => {
                running.store(false, sync::atomic::Ordering::SeqCst);
            }
            _ => (),
        }
    }
    Ok(())
}
fn run_game() -> Result<()> {
    let args = Args::parse();
    let threads = args.threads.unwrap_or_else(|| (num_cpus::get() - 2) as u16);
    let mut infile = std::fs::File::open(args.input)?;
    let initial = read_pgm(&mut infile)?;
    let (sx, tx) = std::sync::mpsc::channel();
    let bsx = sx.clone();
    let running = AtomicBool::new(true);

    let mut curr = initial.clone();
    let running = &running;
    std::thread::scope(move |s| {
        s.spawn(move || {
            mk_pool(threads as usize)
                .expect("failed to create threadpool")
                .install(move || {
                    while running.load(sync::atomic::Ordering::SeqCst) {
                        curr = run_turn(curr, threads as u32).expect("failed to run turn");
                        let r = bsx.send(Event::TurnEnd(curr.clone()));
                        if r.is_err() {
                            break;
                        }
                    }
                })
        });

        run_event_loop(&running, tx, args.background, args.charset)
    })
}
fn with_handler<H, F, R>(handler: H, func: F) -> Result<R, Box<dyn Any + Send>>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
    H: Fn(&PanicInfo) -> () + 'static + Sync + Send,
{
    let old = panic::take_hook();
    panic::set_hook(Box::new(handler));
    let r = panic::catch_unwind(func);
    panic::set_hook(old);
    r
}

fn main() -> Result<()> {
    let panic_buf = Arc::new(Mutex::new(String::new()));
    let rs = with_handler(
        {
            let panic_buf = panic_buf.clone();
            move |info| {
                let mut panic_buf = panic_buf.lock().unwrap();
                panic_buf.push_str(&info.to_string());
            }
        },
        run_game,
    );
    match rs {
        Err(_) => println!("{}", panic_buf.lock().unwrap()),
        Ok(r) => r?,
    };
    Ok(())
}
