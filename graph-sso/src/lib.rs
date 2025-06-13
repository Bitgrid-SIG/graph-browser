#![feature(generic_const_exprs, str_from_raw_parts, ptr_metadata)]
#![allow(incomplete_features)]
#![no_std]

#[macro_use]
mod const_utils;

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate const_str;

use core::mem::MaybeUninit;

/// The number of bytes in each cache-line. This is almost always 32 or 64,
/// with 32 being more common.
pub const CACHELINE_WIDTH: usize = 32;

/// The size of each small-string. This is used to determine the buffer-size
/// of each small-string, how many small-strings will fit into a cache-line,
/// and the amount of padding in each cache-line.
pub const INLINE_CAPACITY: usize = 9;

/// The number of small-strings that can fit into a cache-line.
pub const SS_PER_CACHELINE: usize = CACHELINE_WIDTH / size_of::<SmallString>();

/// The number of bytes used to pad each cache-line.
pub const SS_CACHELINE_PADDING: usize =
    CACHELINE_WIDTH - (SS_PER_CACHELINE * size_of::<SmallString>());

#[derive(core::clone::Clone, core::fmt::Debug)]
pub enum SSErrorType {
    StringTooBig,
    StringEmpty,
    MatchNotFound,
    Uninit,
    Utf8Error(core::str::Utf8Error),
}

type Result<T> = core::result::Result<T, SSErrorType>;

#[repr(C)]
pub struct SmallString(u8, [u8; INLINE_CAPACITY]);

// align(32): even if the width is changed to 64, this is still nicely aligned
// to / against cache-line boundaries
#[repr(C, align(32))]
pub struct SSCacheLine(
    [MaybeUninit<SmallString>; SS_PER_CACHELINE],
    [u8; SS_CACHELINE_PADDING],
);

const_assert_eq!(size_of::<SSCacheLine>(), CACHELINE_WIDTH);

pub struct SmallStringCollection<const N: usize, const L: usize = { N.div_ceil(SS_PER_CACHELINE) }>(
    [SSCacheLine; L],
    usize,
);

impl SmallString {
    pub const fn new(s: &str) -> Result<MaybeUninit<Self>> {
        let l = s.len();

        if l >= INLINE_CAPACITY {
            return Err(SSErrorType::StringTooBig);
        }
        if l == 0 {
            return Err(SSErrorType::StringEmpty);
        }

        let mut bytes: [u8; INLINE_CAPACITY] = [0; INLINE_CAPACITY];
        unsafe { s.as_ptr().copy_to(bytes.as_mut_ptr(), l) };

        Ok(MaybeUninit::new(Self(s.len() as u8, bytes)))
    }

    /// Check if the string has been initialized by checking if the size is
    /// a valid length. Assumes that an invalid length is uninitialized.
    #[inline(always)]
    pub const fn is_init(&self) -> bool {
        self.0 != 0 || self.0 > (INLINE_CAPACITY as u8)
    }

    /// Get a fat pointer (DST) to the buffer using the size of the string
    /// instead of the size of the buffer itself.
    ///
    /// # Safety
    /// This doesn't do any checks so usage needs to be within already-checked
    /// contexts.
    #[inline(always)]
    const unsafe fn get_dst(&self) -> &[u8] {
        unsafe { &*core::ptr::from_raw_parts(self.1.as_ptr(), self.0 as usize) }
    }

    /// Check if the string is valid utf-8, returning an Err if it isn't.
    pub const fn check_utf8(&self) -> Result<()> {
        if self.is_init() {
            // safe because we're in a checked context
            let s: &[u8] = unsafe { self.get_dst() };
            let r = core::str::from_utf8(s);
            if r.is_ok() {
                Ok(())
            } else if let Err(e) = r {
                Err(SSErrorType::Utf8Error(e))
            } else {
                unreachable!()
            }
        } else {
            Err(SSErrorType::Uninit)
        }
    }

    /// Returns the UTF-8 string slice.
    ///
    /// # Safety
    /// A common use of `SmallString` is as a type wrapped in a [`MaybeUninit`].
    ///
    /// This is guaranteed to be safe if it was initialized using [`Self::new`].
    /// If it was initialized any other way (such as [`MaybeUninit::zeroed`]),
    /// there are NO safety guarantees!
    #[inline]
    pub const unsafe fn as_str_unchecked(&self) -> &str {
        unsafe { core::str::from_raw_parts(self.1.as_ptr(), self.0 as usize) }
    }

    /// Returns the UTF-8 string slice.
    ///
    /// # Safety
    /// This function uses [`str::from_utf8`], which checks that the bytes are valid
    /// utf-8. Because this is slow, you should generally prefer [`Self::as_str_maybe`]
    /// for a faster check that's just as reliable.
    ///
    /// See also: [`Self::as_str_unchecked`]
    pub const fn as_str(&self) -> Result<&str> {
        if self.is_init() {
            let s: &[u8] = unsafe { &*core::ptr::from_raw_parts(self.1.as_ptr(), self.0 as usize) };
            let r = core::str::from_utf8(s);
            if let Ok(s) = r {
                Ok(s)
            } else if let Err(e) = r {
                Err(SSErrorType::Utf8Error(e))
            } else {
                unreachable!()
            }
        } else {
            Err(SSErrorType::Uninit)
        }
    }

    /// Like [`Self::as_str`], but returns `None` when called on an uninitialized instance.
    #[inline]
    pub fn as_str_maybe(&self) -> Option<&str> {
        self.is_init().then(|| unsafe { self.as_str_unchecked() })
    }
}

impl SSCacheLine {
    const fn new(ss_arr: &[&str; SS_PER_CACHELINE]) -> Self {
        let mut line: [MaybeUninit<SmallString>; SS_PER_CACHELINE] = unsafe { core::mem::zeroed() };

        let mut l = SS_PER_CACHELINE;
        while l > 0 {
            let r = SmallString::new(ss_arr[l - 1]);
            if let Ok(maybe) = r {
                line[l - 1] = maybe;
                l -= 1;
            } else if let Err(e) = r {
                ss_const_panic!("Failed to create SmallString: ", e);
            }
        }
        let padding: [u8; SS_CACHELINE_PADDING] = [0; SS_CACHELINE_PADDING];
        Self(line, padding)
    }
}

impl<const N: usize, const L: usize> SmallStringCollection<N, L> {
    /// How many bytes are used for padding into cache-line alignment for a given
    /// number of small-strings?
    #[inline(always)]
    pub const fn padding_byte_count_for_ss_count(count: usize) -> usize {
        count * SS_CACHELINE_PADDING
    }

    /// Build a padded cache-line of [`SmallString`]s from an array of `str`s.
    pub const fn new(ss_arr: &[&str; N]) -> Result<Self> {
        let mut lines: [Result<SSCacheLine>; L] = unsafe { core::mem::zeroed() };

        const_loop_range!(0; idx < L; {
            let start = idx * SS_PER_CACHELINE;
            let end = start + SS_PER_CACHELINE;

            lines[idx] = if end < N {
                let span: &[&str; SS_PER_CACHELINE] = unsafe {
                    let unsized_ptr = &ss_arr[start] as *const &str;
                    let resized_ptr = core::ptr::slice_from_raw_parts(unsized_ptr, SS_PER_CACHELINE);
                    let sized_ptr = resized_ptr as *const [&str; SS_PER_CACHELINE];
                    &*sized_ptr
                };

                Ok(SSCacheLine::new(span))
            } else {
                let mut line: [Result<MaybeUninit<SmallString>>; SS_PER_CACHELINE] = unsafe {
                    core::mem::zeroed()
                };

                const_loop_range!(0; i < SS_PER_CACHELINE; {
                    line[i] = if start + i >= N {
                        Ok(MaybeUninit::zeroed())
                    } else {
                        SmallString::new(ss_arr[start + i])
                    };
                });

                let line = const_map_collection!(line.map(r: [Result<MaybeUninit<SmallString>>; SS_PER_CACHELINE]) -> [MaybeUninit<SmallString>; SS_PER_CACHELINE] {
                    if let Ok(s) = r {
                        s
                    } else if let Err(e) = r {
                        ss_const_panic!("Failed to create an SSCacheLine: ", e);
                    } else {
                        unreachable!();
                    }
                });

                Ok(SSCacheLine(line, [0; SS_CACHELINE_PADDING]))
            }
        });

        let lines = const_map_collection!(lines.map(r: [Result<SSCacheLine>; L]) -> [SSCacheLine; L] {
            if let Ok(line) = r {
                line
            } else if let Err(e) = r {
                ss_const_panic!("Failed to create an SSCacheLine: ", e);
            } else {
                unreachable!();
            }
        });

        Ok(Self(lines, N))
    }

    /// Total number of elements in the collection.
    ///
    /// Always returns the const generic `N`.
    #[inline]
    pub const fn size(&self) -> usize {
        N
    }

    pub const fn lookup_idx(&self, idx: usize) -> &MaybeUninit<SmallString> {
        &self.0[idx / SS_PER_CACHELINE].0[idx % SS_PER_CACHELINE]
    }

    /// Sorts the underlying array of small-strings in place using a comparison
    /// function that takes two byte-array references and returns their ordering.
    pub fn sort_with<F: Fn(&[u8; N], &[u8; N]) -> core::cmp::Ordering>(&mut self, _f: F) {
        todo!()
    }

    /// Look up `q` by value and return its `&str` slice on success.
    ///
    /// Errors if:
    /// - `q` is empty ([`StringEmpty`](SSErrorType)),
    /// - `q.len() >= INLINE_CAPACITY` ([`StringTooBig`](SSErrorType)),
    /// - no matching entry ([`MatchNotFound`](SSErrorType)).
    #[inline]
    pub const fn find(&self, q: &str) -> Result<&str> {
        const_map_result!([(let i = self.find_index(q)) => Result<&str>]: {
            unsafe { self.lookup_idx(i).assume_init_ref().as_str_unchecked() }
        })
    }

    /// Look up `q` by value and return its numeric index on success.
    ///
    /// Errors if:
    /// - `q` is empty ([`StringEmpty`](SSErrorType)),
    /// - `q.len() >= INLINE_CAPACITY` ([`StringTooBig`](SSErrorType)),
    /// - no matching entry ([`MatchNotFound`](SSErrorType)).
    pub const fn find_index(&self, s: &str) -> Result<usize> {
        let l = s.len();

        if l >= INLINE_CAPACITY {
            return Err(SSErrorType::StringTooBig);
        } else if l == 0 {
            return Err(SSErrorType::StringEmpty);
        }

        // TODO: Use a more efficient algorithm for finding a matching small-string
        const_loop_range!(0; idx < self.1; {
            let maybe = self.lookup_idx(idx);
            let ss: &SmallString = unsafe { maybe.assume_init_ref() };
            if ss.is_init() {
                let ss_utf = unsafe { ss.as_str_unchecked() };
                if matches!(compare!(ss_utf, s), core::cmp::Ordering::Equal) {
                    return Ok(idx);
                }
            }
        });

        Err(SSErrorType::MatchNotFound)
    }
}

impl core::borrow::Borrow<str> for SmallString {
    fn borrow(&self) -> &str {
        self.as_str_maybe()
            .expect("Tried to borrow &str from an uninitialized SmallString")
    }
}

impl<const N: usize, const L: usize> core::ops::Index<usize> for SmallStringCollection<N, L> {
    type Output = SmallString;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            let output =
                self.0[index / SS_PER_CACHELINE].0[index % SS_PER_CACHELINE].assume_init_ref();
            assert_ne!(output.0, 0, "Indexed SmallString is uninitialized!");
            output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SSC<const T: usize> = SmallStringCollection<T>;

    mod const_tests {
        use super::*;

        mod static_tests {
            use super::*;

            const COL: SSC<3> = const_unwrap_result!(SSC::new(&["a", "b", "c"]));
            const_assert_eq!(COL.0.len(), 1);
            const_assert_eq!(COL.0[0].0.len(), 3);

            // TODO: More const tests
        }

        mod dyn_tests {
            // use super::*;

            // const COL_CONTENTS: &str = include_str!("../const-test-strs.txt");
            // const COL_LINES: [&str; const_str::chain! {}] = split_lines!(COL_CONTENTS);

            // TODO: More const tests
        }
    }

    #[test]
    fn basics() {
        let col: SSC<3> = SSC::new(&["a", "b", "c"]).unwrap();
        assert_eq!(col.0.len(), 1);
        assert_eq!(col.0[0].0.len(), 3);

        assert!(col.find("a").is_ok());
        assert_eq!(col.find("a").unwrap(), "a");

        assert!(col.find("b").is_ok());
        assert_eq!(col.find("b").unwrap(), "b");

        assert!(col.find("c").is_ok());
        assert_eq!(col.find("c").unwrap(), "c");

        assert!(col.find("z").is_err());

        assert!(col.find("").is_err());
        assert!(matches!(
            col.find("").unwrap_err(),
            SSErrorType::StringEmpty
        ));

        let long_string = "a".repeat(INLINE_CAPACITY + 1);
        assert!(col.find(&long_string).is_err());
        assert!(matches!(
            col.find(&long_string).unwrap_err(),
            SSErrorType::StringTooBig
        ));

        // TODO: More tests
    }
}
