#![feature(slice_swap_unchecked)]
#![feature(generic_const_exprs)]
#![feature(int_roundings)]

#![allow(unused)]

pub mod variants;
pub mod hash;
mod util;
mod bytevec;
pub(crate) use bytevec::ByteVec;

pub mod prelude {
    pub use crate::variants::{HashId as _, HashIdDefault as HashIds};
    pub use crate::util::{ToU64Array, DecodeErr};
}