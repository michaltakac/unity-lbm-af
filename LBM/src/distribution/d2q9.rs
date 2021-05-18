use std;
use arrayfire::*;
use crate::FloatNum;
use crate::geometry::Direction;
use crate::traits;

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug)]
#[repr(C)]
pub struct D2Q9 {
    ex: Array<FloatNum>,
    ey: Array<FloatNum>,
    weight: Array<FloatNum>
}

impl D2Q9 {
    #[inline(always)]
    fn new(v: usize) -> D2Q9 {
        assert!(v < 9);
        unsafe { std::mem::transmute(v) }
    }
}

type Iter = std::iter::Map<std::ops::Range<usize>, fn(usize) -> D2Q9>;

impl traits::Distribution for D2Q9 {
    type Storage = [FloatNum; 9];
    type AllIterator = Iter;
    #[inline(always)]
    fn c_squ() -> FloatNum {
        1. / 3.
    }
    #[inline(always)]
    fn size() -> usize {
        9
    }
    #[inline(always)]
    fn dims() -> Dim4 {
        dim4!(Self::size())
    }
    #[inline(always)]
    fn all() -> Self::AllIterator {
        (0..Self::size()).map(D2Q9::new)
    }
    #[inline(always)]
    fn ex() -> Array<FloatNum> {
        Array::<FloatNum>::new(&[0., 1., 0., -1., 0., 1., -1., -1., 1.], Self::dims())
    }
    #[inline(always)]
    fn ey() -> Array<FloatNum> {
        Array::<FloatNum>::new(&[0., 0., 1., 0., -1., 1., 1., -1., -1.], Self::dims())
    }
    #[inline(always)]
    fn weights() -> Array<FloatNum> {
        let t1: FloatNum = 4. / 9.;
        let t2: FloatNum = 1. / 9.;
        let t3: FloatNum = 1. / 36.;
        Array::new(&[t1, t2, t2, t2, t2, t3, t3, t3, t3], Self::dims())
    }
    #[inline(always)]
    fn current_indices(&self) -> Array<FloatNum> {
        (range::<u64>(dim4!(1, 8), 1) + 1) * Self::size()
    }
    #[inline(always)]
    fn opposite_indices(&self) -> Array<FloatNum> {
        (range::<u64>(dim4!(1, 8), 1) + 1) * Self::size()
    }
    #[inline(always)]
    fn direction(&self) -> Direction {
        use self::D2Q9::*;
        match *self {
            C => Direction::C,
            E => Direction::E,
            N => Direction::N,
            W => Direction::W,
            S => Direction::S,
            NE => Direction::NE,
            NW => Direction::NW,
            SW => Direction::SW,
            SE => Direction::SE,
        }
    }

    #[inline(always)]
    fn from_direction(d: Direction) -> Option<Self> {
        use self::D2Q9::*;
        Some(match d {
            Direction::C => C,
            Direction::E => E,
            Direction::N => N,
            Direction::W => W,
            Direction::S => S,
            Direction::NE => NE,
            Direction::NW => NW,
            Direction::SW => SW,
            Direction::SE => SE,
        })
    }
}
