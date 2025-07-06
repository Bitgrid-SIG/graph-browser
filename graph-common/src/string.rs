#[derive(core::clone::Clone, core::fmt::Debug)]
pub enum SSErrorType {
    StringTooBig,
    StringEmpty,
    Malformed,
    Utf8Error(core::str::Utf8Error),
}

type Result<T> = core::result::Result<T, SSErrorType>;

pub struct SString<const N: usize> {
    strlen: u8,
    filled: u8,
    bytes: [u8; N],
}

// empty is specifically disallowed, so implementing it would be redundant.
#[allow(clippy::len_without_is_empty)]
impl<const N: usize> SString<N> {
    pub const SIZEOF: usize = N + 2;

    /// Create a new stack-allocated, cache-friendly string.
    pub const fn new(s: &str) -> Result<Self> {
        let sl = s.len();
        let b = s.as_bytes();
        let bl = b.len();

        if bl >= N {
            return Err(SSErrorType::StringTooBig);
        }
        if sl == 0 {
            return Err(SSErrorType::StringEmpty);
        }

        let mut bytes: [u8; N] = [0; N];
        unsafe { b.as_ptr().copy_to(bytes.as_mut_ptr(), sl) };

        Ok(Self{
            strlen: sl as u8,
            filled: bl as u8,
            bytes
        })
    }

    /// Check if the string has been initialized by checking if the sizes are
    /// valid values.
    #[inline(always)]
    pub const fn is_valid(&self) -> bool {
        0 < self.strlen && self.strlen <= self.filled && self.filled < (N as u8)
    }

    #[inline(always)]
    pub const fn buf_size(&self) -> usize {
        self.filled as usize
    }

    /// Get the length of the &str form of this object. This is different from
    /// the number of bytes occupying the internal buffer. If you're looking
    /// for that, see [`Self::buf_size`].
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.strlen as usize
    }

    /// Get a fat pointer (DST) to the buffer using the size of the string
    /// instead of the size of the buffer itself.
    ///
    /// # Safety
    /// This doesn't do any checks so usage needs to be within already-checked
    /// contexts.
    #[inline(always)]
    const unsafe fn get_dst(&self) -> &[u8] {
        unsafe { &*core::ptr::from_raw_parts(self.bytes.as_ptr(), self.len()) }
    }

    /// Returns the UTF-8 string slice.
    ///
    /// # Safety
    /// A common use of `SmallString` is as a type wrapped in a [`MaybeUninit`].
    ///
    /// This is guaranteed to be safe if it was initialized using [`Self::new`].
    /// If it was initialized any other way (such as [`MaybeUninit::zeroed`]),
    /// there are NO safety guarantees!
    #[inline(always)]
    pub const unsafe fn as_str_unchecked(&self) -> &str {
        unsafe { core::str::from_raw_parts(self.bytes.as_ptr(), self.len()) }
    }

    /// Returns the UTF-8 string slice.
    ///
    /// # Safety
    /// This function uses [`str::from_utf8`], which checks that the bytes are valid
    /// utf-8. Because this is slow, you should generally prefer [`Self::as_str_maybe`]
    /// for a faster check that's just as reliable.
    ///
    /// See also: [`Self::as_str_unchecked`]
    #[inline]
    pub const fn as_str(&self) -> Result<&str> {
        use crate::const_map_result;

        if self.is_valid() {
            // safe because we're in a checked context
            let s: &[u8] = unsafe { self.get_dst() };
            const_map_result!((core::str::from_utf8(s)).map_err(|e| { SSErrorType::Utf8Error(e) }))
        } else {
            Err(SSErrorType::Malformed)
        }
    }

    /// This function uses [`str::from_utf8`] to check whether the bytes are valid
    /// utf-8. Because this is slow, you should generally prefer [`Self::as_str_maybe`]
    /// for a faster check that's just as reliable.
    #[inline]
    pub const fn check_utf8(&self) -> Result<()> {
        use crate::const_map_result;

        const_map_result!((self.as_str()).map(|_| { }))
    }

    /// Like [`Self::as_str`], but returns `None` when called on an uninitialized
    /// instance.
    #[inline]
    pub fn as_str_maybe(&self) -> Option<&str> {
        self.is_valid().then(|| unsafe { self.as_str_unchecked() })
    }
}

impl<const N: usize> std::borrow::Borrow<str> for SString<N> {
    #[inline]
    fn borrow(&self) -> &str {
        unsafe { self.as_str_unchecked() }
    }
}
