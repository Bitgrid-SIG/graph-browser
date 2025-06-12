#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![no_std]

use static_assertions::const_assert_eq;

const CACHELINE_WIDTH: usize = 32;

// try to fit multiple entire small-strings into a standard cache-line
const INLINE_CAPACITY: usize = 9;

const SS_PER_CACHELINE: usize = CACHELINE_WIDTH / size_of::<SmallString>();
const SS_CACHELINE_PADDING: usize = CACHELINE_WIDTH - (SS_PER_CACHELINE * size_of::<SmallString>());

#[derive(core::clone::Clone, core::fmt::Debug)]
pub enum SSErrorType {
    StringTooBig,
    StringEmpty,
    MatchNotFound,
}

type Result<T> = core::result::Result<T, SSErrorType>;

#[repr(C)]
pub struct SmallString(u8, [u8; INLINE_CAPACITY]);

// align(32): even if the width is changed to 64, this is still nicely aligned
// to / against cache-line boundaries
#[repr(C, align(32))]
pub struct SSCacheLine([SmallString; SS_PER_CACHELINE], [u8; SS_CACHELINE_PADDING]);

const_assert_eq!(size_of::<SSCacheLine>(), CACHELINE_WIDTH);

pub struct SmallStringCollection<const N: usize, const L: usize = { N.div_ceil(SS_PER_CACHELINE) }>(
    [SSCacheLine; L],
    usize,
);

fn strcmp(a: &str, b: &str) -> bool {
    let ac = a.chars();
    let bc = b.chars();
    for (a, b) in ac.into_iter().zip(bc.into_iter()) {
        if a != b {
            return false;
        }
    }
    return true;
}

impl SmallString {
    pub fn new<Q: core::borrow::Borrow<str>>(q: &Q) -> Result<Self> {
        let s: &str = q.borrow();
        let l = s.len();

        if l >= INLINE_CAPACITY {
            return Err(SSErrorType::StringTooBig);
        }
        if l == 0 {
            return Err(SSErrorType::StringEmpty);
        }

        let mut bytes: [u8; INLINE_CAPACITY] = [0; INLINE_CAPACITY];
        bytes[..l].copy_from_slice(s.as_bytes());

        Ok(Self(s.len() as u8, bytes))
    }

    /// Returns the UTF-8 string slice.
    ///
    /// # Safety
    /// Guaranteed safe because [`Self::bytes`] is constructed from valid utf-8,
    /// UNLESS this is an [`Self::empty`]. For a checked version, see
    /// [`Self::as_str_checked()`].
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.1[..self.0 as usize]) }
    }

    /// Like [`Self::as_str`], but returns `None` when called on an "empty" zero‐filled instance.
    pub fn as_str_checked(&self) -> Option<&str> {
        (self.0 != 0).then(|| self.as_str())
    }

    /// Create a zero‐filled [`SmallString`].  
    /// 
    /// # Safety
    /// Contents are not valid UTF-8 and length is zero; only use for padding.
    pub unsafe fn empty() -> Self {
        Self(0, [0; INLINE_CAPACITY])
    }
}

impl SSCacheLine {
    fn new<Q: core::borrow::Borrow<str>>(ss_arr: &[Q; SS_PER_CACHELINE]) -> Self {
        let line: [SmallString; SS_PER_CACHELINE] =
            core::array::from_fn(|idx| SmallString::new(&ss_arr[idx]).unwrap());
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
    pub fn new<Q: core::borrow::Borrow<str>>(ss_arr: &[Q; N]) -> Result<Self> {
        let lines: [Result<SSCacheLine>; L] = core::array::from_fn(|idx| {
            let start = idx * SS_PER_CACHELINE;
            let end = start + SS_PER_CACHELINE;

            if end < N {
                let span: &[Q; SS_PER_CACHELINE] = unsafe {
                    let unsized_ptr = &ss_arr[start..end] as *const [Q];
                    let sized_ptr = unsized_ptr as *const [Q; SS_PER_CACHELINE];
                    &*sized_ptr
                };

                Ok(SSCacheLine::new(span))
            } else {
                let line: [Result<SmallString>; SS_PER_CACHELINE] = core::array::from_fn(|i| {
                    ss_arr
                        .get(i)
                        .map_or_else(|| unsafe { Ok(SmallString::empty()) }, SmallString::new)
                });
                
                
                for s in line.iter() {
                    if let Err(e) = s {
                        return Err(e.clone());
                    }
                }

                let line: [SmallString; SS_PER_CACHELINE] = line.map(|o| o.unwrap());
                
                Ok(SSCacheLine(line, [0; SS_CACHELINE_PADDING]))
            }
        });

        for line in lines.iter() {
            if let Err(e) = line {
                return Err(e.clone());
            }
        }

        let lines: [SSCacheLine; L] = lines.map(|o| o.unwrap());

        Ok(Self(lines, N))
    }

    #[inline]
    pub fn find<Q: core::borrow::Borrow<str>>(&self, q: Q) -> Result<&str> {
        self.find_index(q).map(|idx| self[idx].as_str())
    }

    pub fn find_index<Q: core::borrow::Borrow<str>>(&self, q: Q) -> Result<usize> {
        let s: &str = q.borrow();
        let l = s.len();

        if l >= INLINE_CAPACITY {
            return Err(SSErrorType::StringTooBig);
        } else if l == 0 {
            return Err(SSErrorType::StringEmpty);
        }

        // TODO: Use a more efficient algorithm for finding a matching small-string
        (0..self.1).into_iter()
            .find(|&idx| strcmp(self[idx].as_str(), s))
            .ok_or(SSErrorType::MatchNotFound)
    }
}

impl core::borrow::Borrow<str> for SmallString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize, const L: usize> core::ops::Index<usize> for SmallStringCollection<N, L> {
    type Output = SmallString;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.1,
            "Cannot index a SmallStringCollection with length {} using index {}",
            self.1,
            index
        );
        &self.0[index / SS_PER_CACHELINE].0[index % SS_PER_CACHELINE]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SSC<const T: usize> = SmallStringCollection<T>;

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
        assert!(matches!(col.find("").unwrap_err(), SSErrorType::StringEmpty));

        let long_string = "a".repeat(INLINE_CAPACITY + 1);
        assert!(col.find(&long_string as &str).is_err());
        assert!(matches!(col.find(long_string).unwrap_err(), SSErrorType::StringTooBig));
    }
}
