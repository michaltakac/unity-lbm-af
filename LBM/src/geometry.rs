// pub trait Geometry {
//     #[inline(always)]
//     fn contains(&self, x: grid::X) -> bool;
// }
mod circle;

pub use self::circle::Circle;