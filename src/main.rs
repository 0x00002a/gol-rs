use std::io::{BufReader, Read, Write};
use std::thread;

use anyhow::{anyhow, ensure, Result};
use gol::{Mask, Point};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;

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
            for c in cs {
                //dbg!(*c.1, calc_px(&board, &c.0));
                *c.1 = calc_px(&board, &c.0);
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
        .map(|p| if *p.1 { 255 } else { 0 })
        .collect::<Vec<_>>();
    f.write_all(&px)?;
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let threads = args.get(2).and_then(|m| m.parse().ok()).unwrap_or(4);
    let mut infile = std::fs::File::open(args.get(1).expect("no input filename?"))
        .expect("failed to open input file");
    let initial = read_pgm(&mut infile).expect("failed to read pgm input");
    let mut curr = initial;
    let mut turn = 0;
    println!("running");
    mk_pool(threads as usize)
        .expect("failed to create threadpool")
        .install(|| loop {
            curr = run_turn(curr, threads).expect("failed to run turn");
            turn += 1;
            println!("ran turn {} alive {}", turn, curr.alive());
        });
}
