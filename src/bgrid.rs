use std::ops::{Deref, DerefMut};

use crate::gol::{Board, Mask, Point};

pub struct Frame {
    pts: Board,
    view: Mask,
}
pub trait Charset {
    const SCALE: (u32, u32);
    fn encode(&self, bg: char, alive: &Vec<bool>) -> char;
}

pub struct BoxChset {}
impl Charset for BoxChset {
    const SCALE: (u32, u32) = (2, 2);

    fn encode(&self, bg: char, alive: &Vec<bool>) -> char {
        assert_eq!(alive.len() as u32, Self::SCALE.0 * Self::SCALE.1);
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
fn transpose<T>(v: &Vec<T>, w: u32, h: u32) -> Vec<T>
where
    T: Clone,
{
    assert_eq!(v.len() as u32, w * h);
    let mut out = Vec::new();
    out.reserve(v.len());

    for x in 0..w {
        for y in 0..h {
            let val = v[(y * w + x) as usize].clone();
            out.push(val);
        }
    }

    out
}

pub struct BrailleChset {}
impl BrailleChset {
    fn calc_offset(alive: &Vec<bool>) -> u8 {
        transpose(&alive, Self::SCALE.0, Self::SCALE.1)
            .into_iter()
            .enumerate()
            .fold(0, |xs, (n, x)| {
                let x = if x { 1 } else { 0 };
                xs | (x << n)
            })
    }
}
impl Charset for BrailleChset {
    const SCALE: (u32, u32) = (2, 4);

    fn encode(&self, bg: char, alive: &Vec<bool>) -> char {
        assert_eq!(alive.len() as u32, Self::SCALE.0 * Self::SCALE.1);
        let off = Self::calc_offset(&alive) as u32;
        if off == 0 {
            bg
        } else {
            let ch = char::from_u32(0x2800 as u32 + off).unwrap();
            println!("off: {} -> {} (a: {:?})", off, ch, alive);
            ch
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
        let mut bounds = self.view.clone();
        let (scalex, scaley) = C::SCALE;
        bounds.w *= scalex;
        bounds.y *= scaley;
        let bounds = bounds;
        let maxh = self.view.h.min(self.pts.height() / scaley);
        let maxw = self.view.w.min(self.pts.width() / scalex);
        (0..maxh)
            .flat_map(|y| {
                let bounds = &bounds;
                let charset = &charset;
                (0..maxw).map(move |x| {
                    let alive = (0..scaley)
                        .flat_map(|oy| {
                            (0..scalex).map(move |ox| Point {
                                x: (x * scalex + ox + self.view.x) as i64,
                                y: (y * scaley + oy + self.view.y) as i64,
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
        assert_eq!(frame[2].0, Point { x: 0, y: 1 });
        assert_eq!(frame[3].0, Point { x: 1, y: 1 });
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
        assert_eq!(f.render_box().len(), 4);
    }

    #[test]
    fn test_view_smaller_than_remapped_block() {
        let f = Frame::new(
            Board::new(10, (0..100).map(|_| false).collect_vec()),
            Mask {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
            },
        );
        assert_eq!(f.render_box().len(), 4);
    }
    #[test]
    fn test_brailleset() {
        let mut f = empty_frame();
        let render = |f: &Frame| f.render(' ', BrailleChset {})[0].1;
        f[Point { x: 0, y: 0 }] = true;
        assert_eq!(render(&f), '⠁');
        f[Point { x: 0, y: 1 }] = true;
        assert_eq!(render(&f), '⠃');
    }
    #[test]
    fn tranpose_swaps_w_and_h() {
        let initial = vec![0, 0, 1, 1, 2, 2];
        let transed = transpose(&initial, 2, 3);
        assert_eq!(transed, vec![0, 1, 2, 0, 1, 2]);
    }
}
