use arrayfire::*;
use crate::FloatNum;
use crate::geometry;
use crate::grid;

pub trait Simulation {
    fn set_omega(&mut self);
    fn set_initial_conditions(&mut self);
}

pub trait Geometry {
    #[inline(always)]
    fn generate(&self, domain: Array<FloatNum>) -> Array<FloatNum>;
}

pub trait Distribution: Sized + Copy + Sync + Send {
    #[inline(always)]
    fn all() -> Self::AllIterator;
    #[inline(always)]
    fn c_squ() -> FloatNum;
    #[inline(always)]
    fn dims() -> Dim4;
    #[inline(always)]
    fn ex() -> Array<FloatNum>;
    #[inline(always)]
    fn ey() -> Array<FloatNum>;
    #[inline(always)]
    fn weights() -> Array<FloatNum>;
    #[inline(always)]
    fn current_indices(&self) -> Array<FloatNum>;
    #[inline(always)]
    fn opposite_indices(&self) -> Array<FloatNum>;
    #[inline(always)]
    fn direction(&self) -> geometry::Direction;
    #[inline(always)]
    fn from_direction(geom: geometry::Direction) -> Option<Self>;
    #[inline(always)]
    fn size() -> usize;
}

pub type DistributionStorage<D> = <D as Distribution>::Storage;


pub trait Collision<D: Distribution>: Copy + Sync + Send {
    #[inline(always)]
    fn collision<H, IH>(&self, f_hlp: &H, idx_h: IH) -> D::Storage
    where
        IH: Fn(&H, D) -> FloatNum;
}


pub trait Physics: Copy + Sync + Send {
    type Distribution: Distribution;
    #[inline(always)]
    fn collision<FH>(
        &self,
        f_h: &FH,
    ) -> DistributionStorage<Self::Distribution>;
    #[inline(always)]
    fn integral(_: Array<FloatNum>, dims: Dim4) -> Array<FloatNum> {
        constant::<FloatNum>(0.0, dims);
    }

    fn visualize(&self);
}