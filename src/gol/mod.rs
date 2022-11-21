use anyhow::Result;
use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::ChunksMut,
};

#[derive(Clone)]
pub struct Board {
    buf: Vec<bool>,
    width: u32,
}

#[derive(Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone)]
pub struct Mask {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Board {
    pub fn new(width: u32, buf: Vec<bool>) -> Self {
        Board { width, buf }
    }
    fn pt_to_index(&self, mut pt: Point) -> usize {
        if pt.x < 0 {
            pt.x += self.width as i64;
        }
        if pt.x >= self.width as i64 {
            pt.x -= self.width as i64;
        }
        if pt.y < 0 {
            pt.y += self.height() as i64;
        }
        if pt.y > self.height() as i64 {
            pt.y -= self.height() as i64;
        }
        return ((pt.y * self.width as i64) + pt.x) as usize;
    }
    pub fn width(&self) -> u32 {
        return self.width;
    }
    pub fn height(&self) -> u32 {
        return (self.buf.len() / (self.width() as usize)) as u32;
    }
    pub fn neighbors(&self, pt: &Point) -> [bool; 8] {
        let pts = [
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
            Point {
                x: pt.x + 1,
                y: pt.y,
            },
        ];
        return pts.map(|p| self[p]);
    }

    pub fn pixels(&self) -> Vec<(Point, &bool)> {
        let out = Vec::new();
        for i in 0..self.buf.len() {
            out.push((
                Point {
                    x: (i as u32 % self.height()) as i64,
                    y: (i as u32 / self.width()) as i64,
                },
                &self.buf[i],
            ))
        }
        out
    }
    pub fn alive(&self) -> usize {
        self.buf.iter().filter(|v| **v).count()
    }
    pub fn pixels_mut(&mut self) -> Vec<(Point, &mut bool)> {
        let out = Vec::new();
        for i in 0..self.buf.len() {
            out.push((
                Point {
                    x: (i as u32 % self.height()) as i64,
                    y: (i as u32 / self.width()) as i64,
                },
                &mut self.buf[i],
            ))
        }
        out
    }
}

impl Index<Point> for Board {
    type Output = bool;
    fn index(&self, index: Point) -> &Self::Output {
        return &self.buf[self.pt_to_index(index)];
    }
}
impl IndexMut<Point> for Board {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        let idx = self.pt_to_index(index);
        return &mut self.buf[idx];
    }
}
