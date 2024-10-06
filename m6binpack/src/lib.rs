#![cfg_attr(not(test), no_std)]

use core::{
    mem::size_of,
    ops::RangeInclusive,
};

extern crate procmacro; /* to avoid a cargo bug when cross-compiling (e.g. wasm) */

pub use procmacro::unpack;

pub static POINTER_WIDTH: usize = size_of::<usize>();

////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! impl_for_uint {
    ($uint:ty) => {
        impl Unpack<$uint> for $uint {
            fn extract(&self, range: RangeInclusive<usize>) -> $uint {
                let t = size_of::<$uint>() * 8;
                let s = *range.start();
                let e = *range.end();

                debug_assert!(s > 0);
                debug_assert!(e > 0);
                debug_assert!(s <= t);
                debug_assert!(e <= t);

                let mask;

                if e == t {
                    mask = <$uint>::MAX;
                } else {
                    mask = (1 << e) - 1;
                }

                (self & mask) >> (s-1)
            }
        }
    };
    ($($uint:ty),*) => {
        $(
            impl_for_uint!($uint);
        )*
    };
}

impl_for_uint! {
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
}

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Unpack<T> {
    /// default lsb0 radix 1
    fn extract(&self, range: RangeInclusive<usize>) -> T;

    /// msb0
    fn extract_msb(&self, range: RangeInclusive<usize>) -> T {
        let t = size_of::<T>() * 8;
        let s = *range.start();
        let e = *range.end();

        self.extract(t - e + 1..=t - s + 1)
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

// impl Unpack<u32> for u32 {
//     fn extract(&self, range: RangeInclusive<usize>) -> u32 {
//         let t = size_of::<T>() * 8 - 1;
//         let s = *range.start();
//         let e = *range.end();

//         debug_assert!(s <= t);
//         debug_assert!(e <= t);

//         let mask;

//         if e == size_of::<T>() * 8 {
//             mask = u32::MAX
//         } else {
//             mask = (1 << e) - 1;
//         }

//         (self & mask) >> s
//     }
// }


#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_extract_bits() {

        let test_num = 0x8000_0000_0000_000A_u64;

        let res0 = Unpack::extract(&test_num, 64..=64);
        let res1 = Unpack::extract(&test_num, 1..=12);

        assert_eq!(res0, 1);
        assert_eq!(res1, 10);
    }
}
