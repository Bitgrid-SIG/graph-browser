#![no_std]

#[macro_use]
extern crate common;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Fnv32(u32);

impl Fnv32 {
    const OFFSET_BASIS: u32 = 0x811c9dc5;
    const PRIME: u32 = 0x01000193;

    pub const fn hash(byte: u8) -> u32 {
        Self::OFFSET_BASIS ^ byte as u32
    }

    pub const fn hash_slice(bytes: &[u8]) -> u32 {
        let mut byte = Self::OFFSET_BASIS;

        const_loop_range! {
            for i in 0..(bytes.len()).step(1) {
                byte ^= bytes[i] as u32;
                byte = byte.wrapping_mul(Self::PRIME);
            }
        }
        
        byte
    }

    pub const fn new() -> Self {
        Self(Self::OFFSET_BASIS)
    }

    pub const fn update(&mut self, byte: u8) {
        self.0 ^= byte as u32;
        self.0 = self.0.wrapping_mul(Self::PRIME);
    }

    pub const fn update_slice<'a>(&mut self, bytes: &'a [u8]) {
        const_loop_range! {
            for i in 0..(bytes.len()).step(1) {
                self.update(bytes[i]);
            }
        }
    }

    pub fn update_bytes<'a>(&mut self, bytes: impl Into<::core::str::Bytes<'a>>) {
        for b in bytes.into() {
            self.update(b);
        }
    }

    pub const fn finish(self) -> u32 {
        self.0
    }
}

impl ::core::ops::Deref for Fnv32 {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::core::ops::DerefMut for Fnv32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ::core::convert::From<u32> for Fnv32 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl ::core::convert::Into<u32> for Fnv32 {
    fn into(self) -> u32 {
        self.0
    }
}
