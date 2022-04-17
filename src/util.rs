/// Shuffles the ID strings
pub(crate) fn shuffle(values: &mut [u8], salt: &[u8]) {
	if salt.is_empty() {
		return;
	}
	// Explicit: We should *never* wrap, but this avoids
	// potential panics.
	use core::num::Wrapping;
	// Setup loop-pre
	let (mut v, mut p) = (Wrapping(0), Wrapping(0));
	// Change from doing weird shit to doing... well, even weirder shit.
	for i in (1..values.len()).rev().map(|i| Wrapping(i)) {
		v %= salt.len();
		let n = Wrapping(*unsafe {
			// SAFETY: we know exactly how long the salt is anyway, and we're modulating the index by it.
			salt.get_unchecked(v.0)
		} as usize);
		p += n;
		let j = (n + v + p) % i;

		// shuffle
		unsafe {
			// SAFETY: never exceed the total length or even index of the value array.
			values.swap_unchecked(i.0, j.0)
		}
		v += 1;
	}
}

/// Creates a numerically weighted hash
pub(crate) fn make_nhash(values: &[u64]) -> u64 {
	values
		.into_iter()
		.enumerate()
		.map(|(idx, value)| value % (idx as u64 + 100))
		.sum()
}
pub(crate) fn make_hash_fast<const A: usize>(mut val: u64, alph: [u8; A]) -> ([u8; 32], usize) {
	let (mut hash, mut idx) = ([0u8; 32], 32);
	loop {
		idx -= 1;
		unsafe { *hash.get_unchecked_mut(idx) = *alph.get_unchecked(val as usize % A) };
		val /= A as u64;
		if val == 0 {
			return (hash, idx);
		}
	}
}

// ISSUES:
// - alph.iter().position(|&it| it == v)? produces a variable time decode.
// - this lookup table could potentially be done using simd, but currently isn't.
// - abuse of mem::transmute and nonzerou8 to produce a reverse lookup table.
//
// For more efficient and useful option, would use the given alph, extend it, and splat xor
// with a u8x64. Currently, we only use a maximum base of 64, so this would *technically* be sound.
// Unfortunately, I lack the bigbrainedness to do this. If you can figure it out, idk what I can do
// for you, because it wouldn't be enough. Trying to use NEON terms - vqtbl4q_u8 - here, would result
// in other issues, and on avx-512 would have a performance penalty instead. 
// 
// In other news, it's super contrived to get it into the right means.
// EDIT:
// Ok so I have no idea why the lookup table doesn't work, so I give up.
pub(crate) fn unhash<const A: usize>(input: &[u8], alph: [u8; A]) -> Option<u64> {
	//let lookup = unsafe {
	//	let mut lookup = [0u8; 256];
	//	for (x, i) in alph.into_iter().enumerate() {
	//		*lookup.get_unchecked_mut(i as usize) = x as u8;
	//	}
	//	use core::{mem::transmute, num::NonZeroU8 as U8};
	//	let lookup: [Option<U8>; 256] = transmute(lookup);
	//	lookup
	//};
	input.into_iter().enumerate().fold(Some(0), |a, (i, &v)| {
		//let pos = unsafe { *lookup.get_unchecked(v as usize) }?.get() as usize;
		let pos = alph.into_iter().position(|it| it == v)?;
		let b = A.checked_pow((input.len() - i - 1).try_into().ok()?)?;
		let c = pos.checked_mul(b)?;
		a.map(|a| a + c as u64)
	})
}
#[derive(Clone, Copy, Debug)]
pub enum DecodeErr<const N: usize> {
	Value(usize, [u64; N]),
	Hash,
	Items(usize, usize),
}

pub(crate) const fn garbage<T>() -> T {
	unsafe { core::mem::MaybeUninit::uninit().assume_init() }
}
