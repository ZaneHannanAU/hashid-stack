use crate::{bytevec::ByteVec, util, variants::HashId as Hash};
use core::{cmp::Ordering, hint::unreachable_unchecked, marker::PhantomData, num::NonZeroUsize};

#[cfg(all(feature = "smartstring", feature = "std"))]
use smartstring::alias::String;

/// Default bytevec length when returning from an encode function.
/// Set to house option + usize extra within 64B by default.
const BV_L_D: usize = 63 - core::mem::size_of::<usize>();
#[derive(Clone, Copy, Debug)]
pub struct HashId<H: Hash, const SALT: usize,
// const BV_L: usize = BV_L_D
>
where
    [(); H::SEP.len()]: Sized,
    [(); H::REAL - H::GUARDS]: Sized,
{
    /// Internal, set by user. Must have a defined length.
    salt: [u8; SALT],
    min_len: Option<NonZeroUsize>,

    hasher: PhantomData<H>,
    alphabet: [u8; H::REAL - H::GUARDS],
    separators: [u8; H::SEP.len()],
    guards: [u8; H::GUARDS],
}

impl<H: Hash, const SALT: usize,
//const BV_L: usize
> HashId<H, SALT
//, BV_L
>
where
    [(); H::SEP.len()]: Sized,
    [(); H::REAL - H::GUARDS]: Sized,
    [(); H::SEP.len()]: Sized,
    [(); H::ALPH.len()]: Sized,
    [(); H::REAL]: Sized,
{
    /*
    /// Used to set the bytevec length, if needed.
    /// Typically the default (~55) characters is fine, but upping it to 119
    /// for longer lengths of ID, or reducing it to 23 for better memory use
    /// characteristics, is understandable.
    ///
    /// ```rust
    /// # use hashid_stack::prelude::*;
    /// # fn main() {
    /// let idg = HashId::with_salt(b"1 2 3 4")
    ///     .set_bv_len::<23>();
    /// # assert_eq!(idg.encode_buf([1]).unwrap().capacity(), 23)
    /// # }
    /// ```
    pub fn set_bv_len<const N_BV_L: usize>(self) -> HashId<H, SALT, N_BV_L> {
        self
    }
    */
    fn new(
        salt: [u8; SALT],
        min_len: Option<NonZeroUsize>,
        alphabet: [u8; H::REAL - H::GUARDS],
        separators: [u8; H::SEP.len()],
        guards: [u8; H::GUARDS],
    ) -> Self {
        Self {
            salt,
            min_len,
            hasher: PhantomData,
            alphabet,
            separators,
            guards,
        }
    }

    pub(crate) fn init_salt_len(salt: &[u8; SALT], min_len: Option<usize>) -> Self {
        // shortcut for slices
        fn to<const N: usize>(b: &[u8]) -> [u8; N] {
            unsafe { b.try_into().unwrap_unchecked() }
        }
        let mut seps: [_; H::SEP.len()] = to(H::SEP);
        let mut alph: [_; H::REAL] = H::filtered();

        util::shuffle(&mut seps, salt);
        util::shuffle(&mut alph, salt);
        let guards = to(&alph[..H::GUARDS]);
        HashId::new(
            *salt,
            min_len.and_then(NonZeroUsize::new),
            to(&alph[H::GUARDS..]),
            seps,
            guards,
        )
    }
    /// Extends a key to perform setup
    fn extend_key(&self, lottery: u8) -> [u8; H::REAL - H::GUARDS] {
        // Avoid leaving the stack
        let mut tmp: [u8; H::REAL - H::GUARDS] = util::garbage();
        tmp[0] = lottery;

        // seed extension
        for (v, s) in tmp.iter_mut().skip(1).zip(self.salt) {
            *v = s;
        }

        tmp
    }
    /// Reseeds key based on current alphabet state
    fn reseed_key(&self, tmp: &mut [u8; H::REAL - H::GUARDS], alph: &[u8; H::REAL - H::GUARDS]) {
        let alph_start = SALT + 1;
        for (v, s) in tmp.iter_mut().skip(alph_start).zip(alph) {
            *v = *s;
        }
    }

    /// Encode an ID
    ///
    /// ```rust
    /// # use hashid_stack::prelude::*;
    /// # fn main() {
    /// let b1234 = HashIdB64::with_salt(b"1 2 3 4");
    /// println!("{}", b1234.encode_one(1));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    pub fn encode_one(&self, value: u64) -> String {
        let values = core::slice::from_ref(&value);
        match self.encode_buf(values) {
            #[cfg(feature = "smartstring")]
            None => String::new_const(),
            #[cfg(not(feature = "smartstring"))]
            None => String::new(),
            Some(v) => {
                let v = v.as_ref().to_vec();
                unsafe { std::string::String::from_utf8_unchecked(v) }.into()
            }
        }
    }
    /// Encode an ID list
    ///
    /// ```rust
    /// # use hashid_stack::prelude::*;
    /// # fn main() {
    /// let b1234 = HashIdB64::with_salt(b"1 2 3 4");
    /// println!("{}", b1234.encode([1, 2, 3, 4]));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    pub fn encode(&self, values: impl AsRef<[u64]>) -> String {
        match self.encode_inner(values.as_ref()) {
            #[cfg(feature = "smartstring")]
            None => String::new_const(),
            #[cfg(not(feature = "smartstring"))]
            None => String::new(),
            Some(v) => {
                let v = v.as_ref().to_vec();
                unsafe { std::string::String::from_utf8_unchecked(v) }.into()
            }
        }
    }

    pub fn encode_buf(&self, values: impl AsRef<[u64]>) -> Option<ByteVec<BV_L_D>> {
        self.encode_inner(values.as_ref())
    }
    pub fn encode_inner(&self, values: &[u64]) -> Option<ByteVec<BV_L_D>> {
        match values.as_ref() {
            [] => None,
            values => {
                let nh = util::make_nhash(values);
                let mut buffer = ByteVec::new();

                let i = nh as usize % (H::REAL - H::GUARDS);
                let lottery = *unsafe { self.alphabet.get_unchecked(i) };
                buffer.push(lottery);
                let mut tmp = self.extend_key(lottery);
                let mut alph = self.alphabet.clone();

                for (i, &val) in values.iter().enumerate() {
                    let mut val = val;
                    self.reseed_key(&mut tmp, &alph);
                    util::shuffle(&mut alph, &tmp);
                    let last = util::make_hash_fast(val, alph);
                    buffer.extend(&last.0[last.1..]);
                    if i + 1 < values.len() {
                        val %= unsafe { *last.0.get_unchecked(last.1) } as u64 + i as u64;
                        buffer.push(*unsafe {
                            self.separators.get_unchecked(val as usize % H::SEP.len())
                        });
                    }
                }
                if let Some(len) = self.min_len.map(NonZeroUsize::get) {
                    // Extension round 1
                    if buffer.len() < len {
                        let g_idx = nh as usize + *buffer.get(0)? as usize;
                        let guard = *unsafe { self.guards.get_unchecked(g_idx % H::GUARDS) };
                        buffer.insert(0, guard);

                        // Extension round 2
                        if buffer.len() < len {
                            let g_idx = nh as usize + *buffer.get(2)? as usize;
                            let guard = *unsafe { self.guards.get_unchecked(g_idx % H::GUARDS) };
                            buffer.push(guard);
                        }
                    }
                    let mid = (H::REAL - H::GUARDS) / 2;
                    while buffer.len() < len {
                        let (l, r) = alph.split_at(mid);
                        buffer = [r, buffer.as_ref(), l]
                            .into_iter()
                            .flatten()
                            .copied()
                            .collect();

                        let excess = buffer.len() - len;
                        if excess > 0 {
                            let marker = excess / 2;
                            let buf = &buffer.as_ref()[marker..marker + len];
                            buffer = buf.into_iter().copied().collect();
                        }
                    }
                }

                Some(buffer)
            }
        }
    }

    /// Decodes a value
    ///
    /// ```rust
    /// # use hashid_stack::prelude::*;
    /// let gen = HashIdB64::with_salt(b"1 2 3 4");
    ///
    /// ```
    pub fn decode<const OUT: usize>(
        &self,
        input: impl AsRef<[u8]>,
    ) -> Result<[u64; OUT], util::DecodeErr> {
        let input = input.as_ref();
        let out = self.decode_fast(input)?;
        let encode = self.encode_buf(out);
        if let Some(encoded) = encode {
            if encoded.as_ref() == input {
                return Ok(out);
            }
        }
        Err(util::DecodeErr::Hash)
    }

    pub fn decode_fast<const OUT: usize>(
        &self,
        input: impl AsRef<[u8]>,
    ) -> Result<[u64; OUT], util::DecodeErr> {
        self.decode_inner(input.as_ref())
    }
    fn decode_inner<const OUT: usize>(&self, input: &[u8]) -> Result<[u64; OUT], util::DecodeErr> {
        let mut val = input.as_ref();
        if let Some(g_idx) = val.iter().position(|u| self.guards.contains(u)) {
            val = &val[(g_idx + 1)..];
        }
        if let Some(g_idx) = val.iter().rposition(|u| self.guards.contains(u)) {
            val = &val[..g_idx];
        }
        if val.len() < 2 {
            Err(util::DecodeErr::Hash)
        } else {
            let mut alph = self.alphabet.clone();
            match val.split_first() {
                None => unsafe { unreachable_unchecked() },
                Some((&lottery, val)) => {
                    let mut tmp = self.extend_key(lottery);
                    let segs = val.split(|u| self.separators.contains(u));
                    let result = segs.map(|seg| {
                        self.reseed_key(&mut tmp, &alph);
                        util::shuffle(&mut alph, &tmp);
                        util::unhash(seg, alph)
                    });
                    let mut out = [0; OUT];
                    let mut max = 0;
                    for (idx, val) in result.enumerate() {
                        max = idx;
                        if let Some(val) = val {
                            if let Some(o) = out.get_mut(idx) {
                                *o = val;
                            }
                        } else {
                            return Err(util::DecodeErr::Value);
                        }
                    }
                    match max.cmp(&OUT) {
                        Ordering::Equal => Ok(out),
                        _ => Err(util::DecodeErr::Items),
                    }
                }
            }
        }
    }
}
