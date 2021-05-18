use crate::streaming::{stream_2d, stream_3d};
use crate::traits::Distribution;
use arrayfire::*;

/// Rectangular grid of up to three dimensions.
#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct StructuredRectangular<D: Distribution> {
    pub dimensions: Dim4,
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl<D: Distribution> StructuredRectangular {
    #[inline(always)]
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        let dimensions = match z {
            0 => dim4!(x, y),
            1 => dim4!(x, y),
            _ => dim4!(x, y, z),
        };
        StructuredRectangular {
            x,
            y,
            z,
            dimensions,
        }
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        match self.dimensions.ndim() {
            2 => self.x * self.y,
            3 => self.x * self.y * self.z,
        }
    }

    #[inline(always)]
    pub fn main_index(&self) -> Array<u64> {
        moddims(
            &range(dim4!(self.size() * D::size()), 0),
            dim4!(self.x, self.y, D::size()),
        )
    }

    /// Returns the neighboring indices
    #[inline(always)]
    pub fn neighbors_index(&self) -> Array<u64> {
        match self.dimensions {
            2 => flat(&stream_2d(self.main_index())),
            3 => flat(&stream_3d(self.main_index())),
        }
    }
}
