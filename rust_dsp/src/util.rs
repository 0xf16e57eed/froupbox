use std::simd::f32x4;

#[derive(Default, Clone)]
pub struct Interpolator<T: Zippable> {
    val: T,
    diff: T,
}
impl<T: Zippable> Interpolator<T> {
    pub fn next(&mut self) -> T {
        let new = self.val.zip(&self.diff, |x, y| x + y);
        std::mem::replace(&mut self.val, new)
    }
}
impl Interpolator<f32> {
    pub fn next_simd(&mut self) -> f32x4 {
        f32x4::from_array([self.next(), self.next(), self.next(), self.next()])
    }
}

pub trait Zippable: Sized {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self;
}
impl Zippable for f32 {
    fn zip(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        f(*self, *other)
    }
}

pub fn interpolate<T: Zippable>(run_length: f32, start: T, end: T) -> Interpolator<T> {
    Interpolator {
        diff: end.zip(&start, |x, y| (x - y) / run_length),
        val: start,
    }
}
