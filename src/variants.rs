mod sealed {
	/// Sealing implementation - don't implement this if you don't know what you're doing
	pub trait Sealed {}
}
use sealed::Sealed;

use crate::hash;

// I have ZERO IDEA what this constant is about.
const GUARD_DIV: usize = 12;

/// Main implementation for hash IDs.
pub trait HashId: Sized + Sealed {
	/// Alphabet
	const ALPH: &'static [u8];
	/// Separators
	const SEP: &'static [u8];

	/// Derived - real alphabet
	const REAL: usize = Self::ALPH.len() - Self::SEP.len();
	/// Derived - guard constants
	const GUARDS: usize = Self::REAL.div_ceil(GUARD_DIV);
	/// SAFETY: Implemented internally only. Produces a sanitised alphabet.
	fn filtered() -> [u8; Self::REAL] {
		use crate::bytevec::ByteVec;
		let b: ByteVec<{ Self::REAL }>;
		b = Self::ALPH
			.iter()
			.filter(|c| !Self::SEP.contains(c))
			.copied()
			.collect();
		unsafe { b.try_into().unwrap_unchecked() }
	}
	/// Generates a hashid instance using the given salt
	fn with_salt<const SALT: usize>(salt: &[u8; SALT]) -> hash::HashId<Self, SALT>
	where
		[(); Self::SEP.len()]: Sized,
		[(); Self::ALPH.len()]: Sized,
		[(); Self::REAL]: Sized,
		[(); Self::REAL - Self::GUARDS]: Sized,
	{
		hash::HashId::init_salt_len(salt, None)
	}
	/// Generates a hashid instance using the given salt and length
	fn with_salt_and_len<const SALT: usize>(
		salt: &[u8; SALT],
		min_len: impl Into<Option<usize>>,
	) -> hash::HashId<Self, SALT>
	where
		[(); Self::SEP.len()]: Sized,
		[(); Self::ALPH.len()]: Sized,
		[(); Self::REAL]: Sized,
		[(); Self::REAL - Self::GUARDS]: Sized,
	{
		hash::HashId::init_salt_len(salt, min_len.into())
	}
}
/// Generic HashID implementation, using full alphabet
#[derive(Debug, Clone, Copy)]
pub struct HashIdDefault;
impl Sealed for HashIdDefault {}
impl HashId for HashIdDefault {
	const ALPH: &'static [u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
	const SEP: &'static [u8] = b"cfhistuCFHISTU";
}
/// QR-code friendly Hash ID entry
///
/// If everything is uppercase, then this will generate a thin QR code, which can
/// be significantly smaller than any other code here.
#[derive(Debug, Clone, Copy)]
pub struct HashIdQr;
impl Sealed for HashIdQr {}
impl HashId for HashIdQr {
	// Technically this isn't URL-safe, however
	const ALPH: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
	const SEP: &'static [u8] = b"CFHISTU";
}

/// Generic HashID implementation, using base 64 URL safe IDs
#[derive(Debug, Clone, Copy)]
pub struct HashIdB64;
impl Sealed for HashIdB64 {}
impl HashId for HashIdB64 {
	const ALPH: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890-_";
	// well wtf this breaks if I don't have at least - or _ in it lol
	const SEP: &'static [u8] = b"CFHISTUcfhistu-_";
}
/// base32 Hash ID entry, using RFC4648
#[derive(Debug, Clone, Copy)]
pub struct HashIdB32;
impl Sealed for HashIdB32 {}
impl HashId for HashIdB32 {
	const ALPH: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
	const SEP: &'static [u8] = b"CFHISTU";
}

macro_rules! tests {
    ($($i:ident),* ) => {
        #[cfg(test)] mod tests {
					trait Sanity {
						/// SAFETY: test status
						fn sanity_alph_sep();
					}
						impl<T: super::HashId> Sanity for T {
							fn sanity_alph_sep() {
								debug_assert!(
									Self::SEP.iter().all(|c| Self::ALPH.contains(c)),
									"All SEParator must be also in ALPHabet, in type {}",
									core::any::type_name::<Self>()
								);
							}
						}
            use super::*;
            #[test] fn sanity() {
                $( <$i as Sanity>::sanity_alph_sep();)*
            }
            #[test] fn init() {
                $(
                    dbg!(<$i as HashId>::with_salt(b"1234"));
                )*
            }
        }
    };
}
tests!(HashIdDefault, HashIdQr, HashIdB64, HashIdB32);
