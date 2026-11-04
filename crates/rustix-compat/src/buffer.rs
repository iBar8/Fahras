// SPDX-License-Identifier: MIT

use std::mem::MaybeUninit;

/// The buffer kinds rustix accepts for output. Nothing on Windows fills them
/// (extended attributes are unsupported), so only the output types matter.
pub trait Buffer<T> {
    type Output;
}

impl<T> Buffer<T> for &mut [T] {
    type Output = usize;
}

impl<T, const N: usize> Buffer<T> for &mut [T; N] {
    type Output = usize;
}

impl<T> Buffer<T> for &mut Vec<T> {
    type Output = usize;
}

impl<'a, T> Buffer<T> for &'a mut [MaybeUninit<T>] {
    type Output = (&'a mut [T], &'a mut [MaybeUninit<T>]);
}

pub struct SpareCapacity<'a, T>(#[allow(dead_code)] &'a mut Vec<T>);

impl<T> Buffer<T> for SpareCapacity<'_, T> {
    type Output = usize;
}

pub fn spare_capacity<T>(v: &mut Vec<T>) -> SpareCapacity<'_, T> {
    SpareCapacity(v)
}
