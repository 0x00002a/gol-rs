use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Board<T>
where
    T: Copy,
{
    buf: Vec<T>,
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

impl<T> Board<T>
where
    T: Copy,
{
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
    pub fn neighbors(&self, pt: &Point) -> [T; 8] {
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
    pub fn split(&mut self, nb: u32) -> ChunksMut<Self> {
        let masks = Vec::new();
        let offy = if self.height() < nb {
            1
        } else {
            self.height() / nb
        };
        let offx = if self.height() < nb {
            self.width() / nb
        } else {
            0
        };
        let slicew = if self.height() < nb {
            offx
        } else {
            self.width()
        };
        let mut next_mask = Mask {
            x: 0,
            y: 0,
            w: slicew,
            h: offy,
        };
        for n in 0..nb {
            if (n == nb - 1) {
                if next_mask.x + next_mask.w < self.width() {
                    next_mask.w += self.width() - (next_mask.x + next_mask.w)
                }

                if next_mask.y + next_mask.h < self.height() {
                    next_mask.h += self.height() - (next_mask.y + next_mask.h)
                }
            }
            masks.push(next_mask);
            next_mask.x += offx;
            next_mask.y += offy;
        }
    }
}

impl<T> Index<Point> for Board<T>
where
    T: Copy,
{
    type Output = T;

    fn index(&self, index: Point) -> &Self::Output {
        return &self.buf[self.pt_to_index(index)];
    }
}
impl<T> IndexMut<Point> for Board<T>
where
    T: Copy,
{
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        let idx = self.pt_to_index(index);
        return &mut self.buf[idx];
    }
}
