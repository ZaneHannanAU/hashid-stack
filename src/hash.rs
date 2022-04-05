use crate::{
    util::{self, make_hash__fast, shuffle, unhash},
    variants::HashId as Hash, bytevec::ByteVec,
};
use core::marker::PhantomData;
use std::{num::NonZeroUsize, hint::unreachable_unchecked};
#[derive(Clone, Copy, Debug)]
pub struct HashId<H: Hash, const SALT: usize>
where
    [(); H::SEP.len()]: Sized,
    [(); H::REAL - H::GUARDS]: Sized,
{
    salt: [u8; SALT],
    min_len: Option<NonZeroUsize>,

    hasher: PhantomData<H>,
    alphabet: [u8; H::REAL - H::GUARDS],
    separators: [u8; H::SEP.len()],
    guards: [u8; H::GUARDS],
}

impl<H: Hash, const SALT: usize> HashId<H, SALT>
where
    [(); H::SEP.len()]: Sized,
    [(); H::REAL - H::GUARDS]: Sized,
    [(); H::SEP.len()]: Sized,
    [(); H::ALPH.len()]: Sized,
    [(); H::REAL]: Sized,
{
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

    pub(crate) fn init_salt_len(salt: &[u8; SALT], min_len: impl Into<Option<usize>>) -> Self {
        // shortcut for slices
        fn to<const N: usize>(b: &[u8]) -> [u8; N] {
            unsafe { b.try_into().unwrap_unchecked() }
        }
        let mut seps: [_; H::SEP.len()] = to(H::SEP);
        let mut alph: [_; H::REAL] = unsafe { H::filtered() };

        use crate::util::shuffle;
        shuffle(&mut seps, salt);
        shuffle(&mut alph, salt);
        let guards = to(&alph[..H::GUARDS]);
        HashId::new(
            *salt,
            min_len.into().and_then(NonZeroUsize::new),
            to(&alph[H::GUARDS..]),
            seps,
            guards,
        )
    }
    fn extend_key(&self, lottery: u8) -> [u8; H::REAL - H::GUARDS] {
        // Avoid leaving the stack
        let mut tmp: [u8; H::REAL - H::GUARDS] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        tmp[0] = lottery;

        // seed extension
        for (v, s) in tmp.iter_mut().skip(1).zip(self.salt) {
            *v = s;
        }

        tmp
    }
    fn reseed_key(&self, tmp: &mut [u8; H::REAL - H::GUARDS], alph: [u8; H::REAL - H::GUARDS]) {

        let alph_start = SALT + 1;
        for (v, s) in tmp
            .iter_mut()
            .skip(alph_start)
            .take(alph_start - alph.len())
            .zip(alph)
        {
            *v = s;
        }
    }

    pub fn encode<T: util::ToU64Array>(&self, values: T) -> Option<Vec<u8>>
    where
        [(); T::LEN]: Sized,
    {
        if let Some(values) = values.array_nonempty() {
            let nh = util::make_nhash(values);
            let mut buffer = Vec::with_capacity(23);

            let i = nh as usize % (H::REAL - H::GUARDS);
            let lottery = *unsafe { self.alphabet.get_unchecked(i) };
            buffer.push(lottery);
            let mut tmp = self.extend_key(lottery);
            let mut alph = self.alphabet.clone();

            let alph_start = SALT + 1;
            for (i, &val) in values.iter().enumerate() {
                let mut val = val;
                self.reseed_key(&mut tmp, alph);
                shuffle(&mut alph, &tmp);
                let last = make_hash__fast(val, alph);
                buffer.extend_from_slice(&last.0[last.1..]);
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
                    buffer = [r, &buffer, l].into_iter().flatten().copied().collect();


                    let excess = buffer.len() - len;
                    if excess > 0 {
                        let marker = excess / 2;
                        buffer = buffer[marker..marker + len].to_vec();
                    }
                }
            }

            Some(buffer)
        } else {
            None
        }
    }

    pub fn decode<const OUT: usize>(&self, input: impl AsRef<[u8]>) -> Result<[u64; OUT], util::DecodeErr> {
        let out = self.decode_fast(input)?;
        let encode = self.encode(out);
        if let Some(encoded) = encode {
            if encoded == input.as_ref() {
                return Ok(out)
            }
        }
        Err(util::DecodeErr::Hash)
    }

    pub fn decode_fast<const OUT: usize>(&self, input: impl AsRef<[u8]>) -> Result<[u64; OUT], util::DecodeErr> {
        let mut val = input.as_ref();
        if let Some(g_idx) = val.iter().position(|u| self.guards.contains(u)) {
            val = &val[(g_idx+1) ..];
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
                        self.reseed_key(&mut tmp, alph);
                        shuffle(&mut alph, &tmp);
                        unhash(seg, alph)
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
                            return Err(util::DecodeErr::Value)
                        }
                    }
                    match max {
                        OUT => Ok(out),
                        x if x < OUT => Err(util::DecodeErr::TooFew),
                        x if x > OUT => Err(util::DecodeErr::TooMany),
                    }
                }
            }
        }
        

    }
}
