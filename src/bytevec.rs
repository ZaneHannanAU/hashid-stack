use core::{convert::TryInto, iter::*};
use std::mem::MaybeUninit;
use std::num::Wrapping;
pub(crate) struct ByteVec<const N: usize> {
    data: [u8; N],
    idx: Wrapping<usize>,
}

impl<const N: usize> ByteVec<N> {
    fn push(&mut self, b: u8) {
        if let Some(i) = self.data.get_mut(self.idx.0) {
            *i = b;
        }
        self.idx += Wrapping(1);
    }
}

impl<const N: usize> FromIterator<u8> for ByteVec<N> {
    fn from_iter<I: IntoIterator<Item = u8>>(i: I) -> Self {
        let mut bytes = [0; N];
        let mut max = 0;
        for (index, v) in i.into_iter().enumerate() {
            max = index;
            if let Some(i) = bytes.get_mut(index) {
                *i = v;
            }
        }
        Self {
            data: bytes,
            idx: Wrapping(max),
        }
    }
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
