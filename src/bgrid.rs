use std::ops::{Deref, DerefMut};

use crate::gol::{Board, Mask, Point};

pub struct Frame {
    pts: Board,
    view: Mask,
}

impl Frame {
    pub fn new(pts: Board, view: Mask) -> Self {
        Self { pts, view }
    }

    pub fn render(&self) -> Vec<(Point, char)> {
        (0..self.view.bottom())
            .flat_map(|y| {
                (0..self.view.right()).map(move |x| {
                    let alive = (0..2)
                        .flat_map(|oy| {
                            (0..2).map(move |ox| Point {
                                x: (x * 2 + ox) as i64,
                                y: (y * 2 + oy) as i64,
                            })
                        })
                        .map(|p| {
                            if !self.view.contains(&p) {
                                false
                            } else {
                                self.pts[p]
                            }
                        })
                        .collect::<Vec<_>>();
                    // 0: top left
                    // 1: top right
                    // 2: bot left
                    // 3: bot right
                    let ch = if alive.iter().all(|a| *a) {
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
                        '░'
                    };
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
    const empty: [bool; 16] = [
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false,
    ];
    fn empty_frame() -> Frame {
        Frame::new(Board::new(4, empty.into()))
    }
    #[test]
    fn test_individual() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 0 }] = true;
        assert_eq!(f.render().len(), 4);
        assert_eq!(f.render()[0].1, '▘');

        f[Point { x: 1, y: 1 }] = true;
        assert_eq!(f.render()[0].1, '▚');

        f[Point { x: 1, y: 0 }] = true;
        assert_eq!(f.render()[0].1, '▜');
        f[Point { x: 1, y: 0 }] = false;

        f[Point { x: 0, y: 1 }] = true;
        assert_eq!(f.render()[0].1, '▙');
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
                    f.render()[0].1,
                    ' ',
                    "defined char for point combo: {:?}",
                    pts
                );
            } else {
                assert_eq!(f.render()[0].1, ' ');
            }
        }
    }
    #[test]
    fn test_bot_right() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 1 }] = true;
        f[Point { x: 1, y: 0 }] = true;
        f[Point { x: 1, y: 1 }] = true;
        assert_eq!(f.render()[0].1, '▟');
    }
    #[test]
    fn test_compress() {
        let mut f = empty_frame();
        f[Point { x: 0, y: 0 }] = true;
        f[Point { x: 2, y: 0 }] = true;
        let frame = f.render();
        assert_eq!(frame[0].0, Point { x: 0, y: 0 });
        assert_eq!(frame[1].0, Point { x: 1, y: 0 });
        assert_eq!(frame[2].0, Point { x: 2, y: 0 });
        assert_eq!(frame[3].0, Point { x: 3, y: 0 });
    }
}
