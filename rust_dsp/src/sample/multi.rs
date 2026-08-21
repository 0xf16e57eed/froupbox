use crate::Sample;
use std::{array, ops};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Multisample<T: Sample, const N: usize>(pub [T; N]);

impl<T: Sample, const N: usize> Default for Multisample<T, N> {
    fn default() -> Self {
        Self::ZERO
    }
}
impl<T: Sample, const N: usize> Sample for Multisample<T, N> {
    const ZERO: Self = Self([T::ZERO; N]);
    fn sanitize_finite(self) -> Self {
        Self(self.0.map(T::sanitize_finite))
    }
    fn is_silent_below(self, threshold: f32) -> bool {
        self.0.iter().all(|t| t.is_silent_below(threshold))
    }
}

macro_rules! multisample_arithmetic_impls {
    ($((
        $binop_name:ident::$binop_func:ident: $binop:tt,
        $assign_name:ident::$assign_func:ident: $assign:tt)
    )+) => {
        $(
            impl<T: Sample, const N: usize> ops::$binop_name<Self> for Multisample<T, N>
                where T: ops::$binop_name<T, Output = T>
            {
                type Output = Self;
                fn $binop_func(self, rhs: Self) -> Self {
                    Self(array::from_fn(|i| self.0[i] $binop rhs.0[i]))
                }
            }
            impl<T: Sample, const N: usize> ops::$binop_name<f32> for Multisample<T, N>
                where T: ops::$binop_name<f32, Output = T>
            {
                type Output = Self;
                fn $binop_func(self, rhs: f32) -> Self {
                  Self(array::from_fn(|i| self.0[i] $binop rhs))
                }
            }
            impl<T: Sample, const N: usize> ops::$assign_name<Self> for Multisample<T, N>
                where T: ops::$assign_name<T>
            {
                fn $assign_func(&mut self, rhs: Self) {
                  for i in 0..N {
                    self.0[i] $assign rhs.0[i];
                  }
                }
            }
            impl<T: Sample, const N: usize> ops::$assign_name<f32> for Multisample<T, N>
                where T: ops::$assign_name<f32>
            {
                fn $assign_func(&mut self, rhs: f32) {
                  for i in 0..N {
                    self.0[i] $assign rhs;
                  }
                }
            }
        )+
    };
}
multisample_arithmetic_impls! {
    (Add::add: +, AddAssign::add_assign: +=)
    (Sub::sub: -, SubAssign::sub_assign: -=)
    (Mul::mul: *, MulAssign::mul_assign: *=)
    (Div::div: /, DivAssign::div_assign: /=)
}
