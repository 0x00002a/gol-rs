use std::io::{BufReader, Read, Write};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use anyhow::{anyhow, ensure, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gol::{Mask, Point};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use std::{io, time::Duration};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans, Text};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Widget},
    Terminal,
};

use crate::render::Renderer;

mod gol;
mod render;

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
    println!("reading pgm");
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
    KeyPress(char),
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let threads = args.get(2).and_then(|m| m.parse().ok()).unwrap_or(4);
    let mut infile = std::fs::File::open(args.get(1).expect("no input filename?"))
        .expect("failed to open input file");
    let initial = read_pgm(&mut infile).expect("failed to read pgm input");
    let mut turn = 0;
    println!("running");
    let (sx, tx) = std::sync::mpsc::channel();
    let bsx = sx.clone();
    thread::scope(move |s| {
        let mut curr = initial.clone();
        let mut running = true;
        s.spawn(move || -> Result<()> {
            mk_pool(threads as usize)
                .expect("failed to create threadpool")
                .install(move || -> Result<()> {
                    while running {
                        curr = run_turn(curr, threads).expect("failed to run turn");
                        //turn += 1;
                        //println!("ran turn {} alive {}", turn, curr.alive());
                        bsx.send(Event::TurnEnd(curr.clone()))?;
                    }
                    Ok(())
                })?;
            Ok(())
        });
        let ksx = sx.clone();
        s.spawn(move || {
            let mut stdin = std::io::stdin();
            let mut buf: [u8; 1] = [0];
            let _ = stdin.lock();
            while running {
                stdin.read_exact(&mut buf).unwrap();
                ksx.send(Event::KeyPress(buf[0] as char)).unwrap();
            }
        });
        //let mut turn = 0;
        //loop {
        //let b = tx.recv().unwrap();
        //turn += 1;
        ////let mut out = std::fs::File::create("out.pgm").unwrap();
        //println!("turn: {}, alive: {}", turn, b.alive());
        ////write_pgm(&b, &mut out).unwrap();
        //}
        enable_raw_mode().expect("failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut offset = Point { x: 0, y: 0 };
        while running {
            let ev = tx.recv().unwrap();
            match ev {
                Event::TurnEnd(b) => {
                    turn += 1;
                    offset.remap(b.width(), b.height());
                    terminal
                        .draw(|f| {
                            let tsize = f.size();
                            for py in 0..tsize.bottom() {
                                for px in 0..tsize.right() {
                                    let pt = Point {
                                        x: px as i64 + offset.x,
                                        y: py as i64 + offset.y,
                                    };
                                    let v = b[pt.clone()];

                                    let c = if v { Color::White } else { Color::Black };
                                    let block = Block::default().style(Style::default().bg(c));
                                    let mut rpt = Rect::default();
                                    rpt.x = px;
                                    rpt.y = py;
                                    rpt.width = 1;
                                    rpt.height = 1;
                                    f.render_widget(block, rpt);
                                }
                            }
                            let turn_count = Block::default().title(format!("turn {}", turn));
                            let mut turn_r = Rect::default();
                            f.render_widget(turn_count, turn_r);
                        })
                        .expect("failed to render frame");
                }
                Event::KeyPress('q') => running = false,
                Event::KeyPress('l') => offset.x += 1,
                Event::KeyPress('h') => offset.x -= 1,
                Event::KeyPress('k') => offset.y -= 1,
                Event::KeyPress('j') => offset.y += 1,
                _ => (),
            }
        }
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        terminal.show_cursor().unwrap();

        /*while r.tick() {
            /*let b = tx.try_recv();
            if b.is_ok() {
                r = r.render_board(&b.unwrap());
            }*/
        }*/
    });
}
