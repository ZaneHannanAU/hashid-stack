use crate::util;
use core::{convert::TryInto, iter::*, num::Wrapping};
pub struct ByteVec<const N: usize> {
    data: [u8; N],
    idx: Wrapping<usize>,
}

pub mod display {
    use core::fmt;
    use std::fmt::Write;

    use super::ByteVec;
    pub struct Display<'a, const N: usize>(pub &'a ByteVec<N>);
    impl<const N: usize> fmt::Display for Display<'_, N> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str(unsafe { core::str::from_utf8_unchecked(self.0.as_ref()) })
        }
    }
    pub struct DisplayChecked<'a, const N: usize>(pub &'a ByteVec<N>);
    impl<const N: usize> fmt::Display for DisplayChecked<'_, N> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.as_ref().iter().try_for_each(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    let c = b as char;
                    f.write_char(c)
                } else {
                    Ok(())
                }
            })
        }
    }
}
impl<const N: usize> ByteVec<N> {
    pub fn new() -> Self {
        Self {
            data: util::garbage(),
            idx: Wrapping(0),
        }
    }

    pub fn push(&mut self, b: u8) {
        if let Some(i) = self.data.get_mut(self.idx.0) {
            *i = b;
        }
        self.idx += Wrapping(1);
    }
    /// SAFETY: takes whatever is in the buffer and dumps it to output unfiltered.
    pub unsafe fn display_unchecked(&self) -> display::Display<N> {
        display::Display(&self)
    }
    /// SAFETY: only displays printable ASCII - 0x20..=0x7E
    pub fn display_ascii(&self) -> display::DisplayChecked<N> {
        display::DisplayChecked(&self)
    }
    pub fn get(&self, off: usize) -> Option<&u8> {
        if off <= self.idx.0 {
            Some(unsafe { self.data.get_unchecked(off) })
        } else {
            None
        }
    }
    pub fn insert(&mut self, idx: usize, value: u8) {
        self.data.copy_within(idx..=self.idx.0, idx + 1);
        unsafe {
            *self.data.get_unchecked_mut(idx) = value;
            self.idx += 1;
        }
    }
    pub fn len(&self) -> usize {
        self.idx.0
    }
    pub fn as_slice(&self) -> &[u8] {
        self.as_ref()
    }
    pub fn capacity() -> usize { N }
}

impl<const N: usize> FromIterator<u8> for ByteVec<N> {
    fn from_iter<I: IntoIterator<Item = u8>>(i: I) -> Self {
        let mut bytes: [u8; N] = util::garbage();
        let mut max = 0;
        let mut iter = i.into_iter();
        for (i, (v, s)) in bytes.iter_mut().zip(&mut iter).enumerate() {
            max = i;
            *v = s;
        }
        max += iter.count();
        Self {
            data: bytes,
            idx: Wrapping(max),
        }
    }
}
impl<const N: usize> Extend<u8> for ByteVec<N> {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, i: I) {
        let mut iter = i.into_iter();
        for (v, s) in (&mut self.data[self.idx.0..]).iter_mut().zip(&mut iter) {
            self.idx += 1;
            *v = s;
        }
        self.idx += iter.count();
    }
    //fn extend_one(&mut self, b: u8) {
    //    self.push(b);
    //}
}
impl<'a, const N: usize> Extend<&'a u8> for ByteVec<N> {
    fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, i: I) {
        let mut iter = i.into_iter();
        for (v, s) in (&mut self.data[self.idx.0..]).iter_mut().zip(&mut iter) {
            self.idx += 1;
            *v = *s;
        }
        self.idx += iter.count();
    }
    //fn extend_one(&mut self, b: &u8) {
    //    self.push(*b);
    //}
}

impl<const N: usize> TryInto<[u8; N]> for ByteVec<N> {
    type Error = (usize, usize);
    fn try_into(self) -> Result<[u8; N], (usize, usize)> {
        if self.idx.0 == (N - 1) {
            Ok(self.data)
        } else {
            Err((self.idx.0, N))
        }
    }
}
impl<const N: usize> AsRef<[u8]> for ByteVec<N> {
    /// SAFETY: technically incorrect.
    fn as_ref(&self) -> &[u8] {
        &self.data[..self.idx.0]
    }
}

impl<const N: usize> AsMut<[u8]> for ByteVec<N> {
    /// SAFETY: technically incorrect.
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.idx.0]
    }
}
