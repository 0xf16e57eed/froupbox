use std::simd::{f32x4, simd_swizzle};

#[derive(Default, Clone, Debug)]
pub struct Interpolator<T: Zippable> {
    pub val: T,
    pub diff: T,
}
impl<T: Zippable> Interpolator<T> {
    pub fn next(&mut self) -> T {
        let new = self.val.zip(&self.diff, |x, y| x + y);
        std::mem::replace(&mut self.val, new)
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

pub fn concat_rotate(s1: &mut f32x4, s2: &mut f32x4) {
    (*s1, *s2) = (
        simd_swizzle!(*s1, *s2, [1, 2, 3, 4]),
        simd_swizzle!(*s1, *s2, [5, 6, 7, 0]),
    );
}
