use anyhow::Result;
use std::{
    fmt::Display,
    ops::{Add, Index, IndexMut},
};

#[derive(Clone, Debug)]
pub struct Board {
    buf: Vec<bool>,
    width: u32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl Add for Point {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}
impl<I1, I2> From<(I1, I2)> for Point
where
    I1: Into<i64>,
    I2: Into<i64>,
{
    fn from((l, r): (I1, I2)) -> Self {
        Self {
            x: l.into(),
            y: r.into(),
        }
    }
}

#[derive(Clone, Debug)]
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
    pub fn contains(&self, pt: &Point) -> bool {
        pt.x >= self.x as i64
            && pt.x <= self.right() as i64
            && pt.y >= self.y as i64
            && pt.y <= self.bottom() as i64
    }
}
impl Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} -> {}, {} -> {})",
            self.x,
            self.right(),
            self.y,
            self.bottom()
        )
    }
}

impl Board {
    pub fn new(width: u32, buf: Vec<bool>) -> Self {
        Board { width, buf }
    }
    fn pt_to_index(&self, mut pt: Point) -> usize {
        let orig = pt.clone();
        pt.remap(self.width(), self.height());
        let index = ((pt.y * self.width as i64) + pt.x) as usize;
        assert!(
            index < (self.width() * self.height()) as usize,
            "pt {} inside bounds (adj: {})",
            pt,
            orig
        );
        index
    }
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
    pub fn remap<T>(&mut self, w: T, h: T)
    where
        T: Into<i64>,
    {
        let w = w.into();
        let h = h.into();
        while self.x < 0 {
            self.x += w;
        }
        while self.y < 0 {
            self.y += h;
        }
        self.x %= w;
        self.y %= h;
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
    #[test]
    fn test_remap() {
        let w = 10 as i64;
        let h = 10 as i64;
        let mut pt = Point { x: w * 2, y: 0 };
        pt.remap(w, h);
        assert!(w * pt.x + pt.y < w * h);

        pt.x = 1;
        pt.y = -2;
        pt.remap(w, h);
        assert_eq!(pt.x, 1);
        assert_eq!(pt.y, h - 2);
    }
}
