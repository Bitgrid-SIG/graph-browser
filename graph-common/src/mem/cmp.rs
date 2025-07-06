
use crate::const_loop_range;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub const fn memcmp_bytes(a: *const u8, b: *const u8, len: usize) -> bool {
    const_loop_range!{
        for i in (0)..(len).step(1) {
            unsafe {
                if *a.add(i) != *b.add(i) {
                    return false;
                }
            }
        }
    };

    true
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub const fn memcmp_aligned(a: *const u8, b: *const u8, byte_len: usize) -> bool {
    let big_ptr_a = a as *const u32;
    let big_ptr_b = b as *const u32;

    let count = byte_len >> 2;
    let tail = byte_len - count;

    const_loop_range!{
        for i in (0)..(count).step(1) {
            unsafe {
                if *big_ptr_a.add(i) != *big_ptr_b.add(i) {
                    return false;
                }
            }
        }
    };

    if tail > 0 {
        const_loop_range!{
            for i in (byte_len - tail)..(byte_len).step(1) {
                unsafe {
                    if *a.add(i) != *b.add(i) {
                        return false;
                    }
                }
            }
        };
    }

    true
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub const fn memcmp_unaligned(a: *const u8, b: *const u8, byte_len: usize, head: usize, tail: usize) -> bool {
    if head > 0 {
        const_loop_range!{
            for i in (0)..(head).step(1) {
                unsafe {
                    if *a.add(i) != *b.add(i) {
                        return false;
                    }
                }
            }
        };
    }

    let big_ptr_a = unsafe { a.add(head) } as *const u32;
    let big_ptr_b = unsafe { b.add(head) } as *const u32;
    let count = (byte_len - head) >> 2;
    
    const_loop_range!{
        for i in (0)..(count).step(1) {
            unsafe {
                if *big_ptr_a.add(i) != *big_ptr_b.add(i) {
                    return false;
                }
            }
        }
    };

    if tail > 0 {
        const_loop_range!{
            for i in (byte_len - tail)..(byte_len).step(1) {
                unsafe {
                    if *a.add(i) != *b.add(i) {
                        return false;
                    }
                }
            }
        };
    }

    true
}

// pub const fn memcmp(a: &[u8], b: &[u8]) -> bool {
//     let l = a.len();

//     if b.len() != l {
//         return false;
//     }

//     let ptr_a = a.as_ptr();
//     let ptr_b = b.as_ptr();

//     const ALIGNMENT: usize = align_of::<usize>();

//     let head = ALIGNMENT - ((ptr_a as usize) % ALIGNMENT);

//     if ALIGNMENT - ((ptr_b as usize) % ALIGNMENT) != head {
//         return memcmp_bytes(ptr_a, ptr_b, l);
//     }

//     let tail = ALIGNMENT - ((ptr_a as usize) + l % ALIGNMENT);

//     if head != 0 || tail != 0 {
//         memcmp_unaligned(ptr_a, ptr_b, l, head, tail)
//     } else {
//         memcmp_aligned(ptr_a, ptr_b, l)
//     }
// }
