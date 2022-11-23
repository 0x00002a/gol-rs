use anyhow::{anyhow, Result};
use rayon::iter::*;
use std::{
    fmt::Display,
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
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
pub struct Mask {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl Mask {
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> u32 {
        self.y + self.h
    }
    pub fn dy(&self) -> u32 {
        self.h.abs_diff(self.y)
    }

    pub fn dx(&self) -> u32 {
        self.w.abs_diff(self.x)
    }
}

impl Board {
    pub fn new(width: u32, buf: Vec<bool>) -> Self {
        Board { width, buf }
    }
    fn pt_to_index(&self, mut pt: Point) -> usize {
        pt.remap(self.width(), self.height());
        return ((pt.y * self.width as i64) + pt.x) as usize;
    }
    pub fn slice(&self, sect: &Mask) -> Result<Self> {
        let mut out = Vec::new();
        out.reserve((sect.w * sect.h) as usize);
        for y in sect.y..sect.bottom() {
            for x in sect.x..sect.right() {
                out.push(
                    self[Point {
                        x: x as i64,
                        y: y as i64,
                    }],
                );
            }
        }
        Ok(Self {
            buf: out,
            width: sect.w,
        })
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

impl Point {
    pub fn remap(&mut self, w: u32, h: u32) {
        if self.x < 0 {
            self.x += w as i64;
        }
        if self.x >= w as i64 {
            self.x -= w as i64;
        }
        if self.y < 0 {
            self.y += h as i64;
        }
        if self.y >= h as i64 {
            self.y -= h as i64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_board() -> Result<()> {
        let init = Board::new(4, (0..16).map(|_| false).collect());
        let mut whole = init.clone();
        whole[Point { x: 1, y: 1 }] = true;
        let sliced = whole.slice(&Mask {
            x: 1,
            y: 1,
            w: 2,
            h: 2,
        })?;
        assert_eq!(sliced[Point { x: 0, y: 0 }], true);
        assert_eq!(sliced.width(), 2);
        assert_eq!(sliced.height(), 2);
        Ok(())
    }
}
