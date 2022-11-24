use std::ops::{Deref, DerefMut};

use crate::gol::{Board, Mask, Point};

pub struct Frame {
    pts: Board,
    view: Mask,
}
pub trait Charset {
    const SCALE: usize;
    fn encode(&self, background: char, pts: &Vec<bool>) -> char;
}

pub struct BoxChset {}
impl Charset for BoxChset {
    const SCALE: usize = 2;

    fn encode(&self, bg: char, alive: &Vec<bool>) -> char {
        assert_eq!(alive.len(), Self::SCALE * 2);
        if alive.iter().all(|a| *a) {
            '█'
        } else if alive[0] && alive[1] && alive[2] {
            '▛'
        } else if alive[0] && alive[1] && alive[3] {
            '▜'
        } else if alive[0] && alive[2] && alive[3] {
            '▙'
        } else if alive[1] && alive[2] && alive[3] {
            '▟'
        } else if alive[0] && alive[1] {
            '▀'
        } else if alive[2] && alive[3] {
            '▄'
        } else if alive[1] && alive[3] {
            '▐'
        } else if alive[0] && alive[2] {
            '▌'
        } else if alive[0] && alive[3] {
            '▚'
        } else if alive[1] && alive[2] {
            '▞'
        } else if alive[0] {
            '▘'
        } else if alive[1] {
            '▝'
        } else if alive[2] {
            '▖'
        } else if alive[3] {
            '▗'
        } else {
            bg
        }
    }
}
type Rendered = Vec<(Point, char)>;

impl Frame {
    pub fn new(pts: Board, view: Mask) -> Self {
        Self { pts, view }
    }
    #[allow(dead_code)]
    pub fn render_box(&self) -> Rendered {
        self.render(' ', BoxChset {})
    }

    pub fn render<C>(&self, background: char, charset: C) -> Rendered
    where
        C: Charset,
    {
        let bounds = self.view.clone().scale(C::SCALE as u32);
        let maxh = self.view.h.min(self.pts.width());
        let maxw = self.view.w.min(self.pts.width());
        (0..maxh)
            .flat_map(|y| {
                let bounds = &bounds;
                let charset = &charset;
                (0..maxw).map(move |x| {
                    let alive = (0..C::SCALE as u32)
                        .flat_map(|oy| {
                            (0..C::SCALE as u32).map(move |ox| Point {
                                x: (x * 2 + ox + self.view.x) as i64,
                                y: (y * 2 + oy + self.view.y) as i64,
                            })
                        })
                        .map(|p| {
                            if !bounds.contains(&p) {
                                false
                            } else {
                                self.pts[p]
                            }
                        })
                        .collect::<Vec<_>>();
                    let ch = charset.encode(background, &alive);
                    (
                        Point {
                            x: x as i64,
                            y: y as i64,
                        },
                        ch,
                    )
                })
            })
            .collect()
    }
}

impl Deref for Frame {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.pts
    }
}
impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pts
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::*;
    const EMPTY: [bool; 16] = [
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false,
    ];
    fn empty_frame() -> Frame {
        Frame::new(
            Board::new(4, EMPTY.into()),
            Mask {
                x: 0,
                y: 0,
                w: 4,
                h: 4,
            },
        )
    }
    #[test]
    fn test_individual() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 0 }] = true;
        assert_eq!(f.render_box().len(), 16);
        assert_eq!(f.render_box()[0].1, '▘');

        f[Point { x: 1, y: 1 }] = true;
        assert_eq!(f.render_box()[0].1, '▚');

        f[Point { x: 1, y: 0 }] = true;
        assert_eq!(f.render_box()[0].1, '▜');
        f[Point { x: 1, y: 0 }] = false;

        f[Point { x: 0, y: 1 }] = true;
        assert_eq!(f.render_box()[0].1, '▙');
        f[Point { x: 0, y: 1 }] = false;
    }
    #[test]
    fn test_all_defined() {
        for pts in (0..2)
            .flat_map(|y| {
                (0..2).map(move |x| Point {
                    x: x as i64,
                    y: y as i64,
                })
            })
            .powerset()
        {
            let mut f = empty_frame();
            for pt in &pts {
                f[pt.clone()] = true;
            }
            if pts.len() > 0 {
                assert_ne!(
                    f.render_box()[0].1,
                    ' ',
                    "defined char for point combo: {:?}",
                    pts
                );
            } else {
                assert_eq!(f.render_box()[0].1, ' ');
            }
        }
    }
    #[test]
    fn test_bot_right() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 1 }] = true;
        f[Point { x: 1, y: 0 }] = true;
        f[Point { x: 1, y: 1 }] = true;
        assert_eq!(f.render_box()[0].1, '▟');
    }
    #[test]
    fn test_compress() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 0 }] = true;
        f[Point { x: 2, y: 0 }] = true;
        let frame = f.render_box();
        assert_eq!(frame[0].0, Point { x: 0, y: 0 });
        assert_eq!(frame[1].0, Point { x: 1, y: 0 });
        assert_eq!(frame[2].0, Point { x: 2, y: 0 });
        assert_eq!(frame[3].0, Point { x: 3, y: 0 });
    }
    #[test]
    fn test_smaller_frame_than_mask() {
        let f = Frame::new(
            Board::new(4, EMPTY.into()),
            Mask {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            },
        );
        assert_eq!(f.render_box().len(), 16);
    }
}
