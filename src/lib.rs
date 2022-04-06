// used in util::shuffle extensively
#![feature(slice_swap_unchecked)]
// used in hash::HashId, variants::HashId extensively
#![feature(generic_const_exprs)]
// used in variants::HashId for guard info.
#![feature(int_roundings)]
// used in bytevec::ByteVec as core::iter::Extend
//#![feature(extend_one)]
// used in hash::HashIds::set_bv_len
//#![feature(type_changing_struct_update)]
#![cfg_attr(not(feature = "std"), no_std)]
//#![allow(unused)]
//#![allow(incomplete_features)]

//! # HashID_stack
//!
//! Heavily based off of [`harsh`](https://lib.rs/crates/harsh), this project
//! takes the basic layout of it and turns it into a somewhat more performant ideal.
//!
//! Some changes:
//!
//! - Limited to a default set of variants (hashid, QR-friendly, base64-url, base32)
//! - Majority of work is done at compile time
//! - All structures are `Copy`.
//! - Compatible with `no_std`.
//!
//! Some downsides:
//!
//! - Currently requires nightly compiler due to use of
//!     - `generic_const_exprs` for most functionality
//!     - `int_roundings` for guard use
//!     - `slice_swap_unchecked` for `util::shuffle`
//!     - `extend_one` for `bytevec::ByteVec<N> as core::iter::Extend<u8>`
//! - Less tested
//! - All inbound data must be known at compile time, including length of the salt.
//!
//! This does come with some benefits, though
//!
//! - Output binaries may be much smaller
//! - Little to no requirement for allocation
//!

pub mod hash;
mod util;
pub mod variants;

pub mod prelude {
	pub use crate::util::DecodeErr;
	pub use crate::variants::{HashId as _, HashIdDefault as HashIds, *};
}

/// Simple `Copy` byte vector. Has display.
pub mod bytevec;
