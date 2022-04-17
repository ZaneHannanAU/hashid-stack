// needed for feature(generic_const_exprs)
// see https://github.com/rust-lang/rust/issues/76560 for more info
#![allow(incomplete_features)]

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
//! - Less tested
//! - All inbound data must be known at compile time, including length of the salt.
//! - The return values are an array that have some custom formatting functions added, or a [`SmartString<LazyCompact>`](https://docs.rs/smartstring/latest/smartstring/alias/type.String.html).
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
	pub use crate::variants::{
		HashId as _, HashIdB32 as B32, HashIdB64 as B64, HashIdDefault as HashIds,
		HashIdDefault as Normal, HashIdQr as QR, *,
	};
	pub use crate::hash::HashId as HashID;
}

/// Simple `Copy` byte vector. Has display.
pub mod bytevec;
