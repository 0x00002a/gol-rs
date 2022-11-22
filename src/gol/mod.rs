use anyhow::Result;
use rayon::iter::*;
use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::ChunksMut,
};

#[derive(Clone, Debug)]
pub struct Board {
    buf: Vec<bool>,
    width: u32,
}

#[derive(Clone, Debug)]
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
        if pt.y >= self.height() as i64 {
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
                x: pt.x - 1,
                y: pt.y,
            },
            Point {
                x: pt.x,
                y: pt.y + 1,
            },
            Point {
                x: pt.x,
                y: pt.y - 1,
            },
            Point {
                x: pt.x + 1,
                y: pt.y - 1,
            },
            Point {
                x: pt.x + 1,
                y: pt.y + 1,
            },
            Point {
                x: pt.x - 1,
                y: pt.y + 1,
            },
            Point {
                x: pt.x - 1,
                y: pt.y - 1,
            },
        ];
        pts.map(|p| self[p])
    }

    pub fn pixels(&self) -> Vec<(Point, bool)> {
        self.buf
            .iter()
            .enumerate()
            .map(|(i, b)| {
                (
                    Point {
                        x: (i as u32 % self.height()) as i64,
                        y: (i as u32 / self.width()) as i64,
                    },
                    *b,
                )
            })
            .collect()
    }
    pub fn alive(&self) -> usize {
        self.buf.iter().filter(|v| **v).count()
    }
    pub fn pixels_mut(&mut self) -> Vec<(Point, &mut bool)> {
        let h = self.height();
        let w = self.width();
        self.buf
            .iter_mut()
            .enumerate()
            .map(|(i, b)| {
                (
                    Point {
                        x: (i as u32 % h) as i64,
                        y: (i as u32 / w) as i64,
                    },
                    b,
                )
            })
            .collect()
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
