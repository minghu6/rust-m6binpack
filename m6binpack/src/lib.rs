#![cfg_attr(not(test), no_std)]

use core::mem::{
    size_of
};


extern crate procmacro; /* to avoid a cargo bug when cross-compiling (e.g. wasm) */

pub use procmacro::{
    unpack
};

pub static POINTER_WIDTH: usize = size_of::<usize>();

/// Extract bit from usize
/// [bitstart, bitstart + bitlen)
pub fn extract_bits(target: usize, bitstart: usize, bitlen: usize) -> usize {
    let bitend = bitstart + bitlen;

    let mask: usize;
    if bitend == POINTER_WIDTH * 8 {
        mask = usize::MAX;
    } else {
        mask = (1 << bitend) - 1;
    }

    (target & mask) >> bitstart
}


#[cfg(test)]
mod test {
    #[test]
    fn test_extract_bits() {
        use crate::extract_bits;

        let test_num = 0x8000_0000_0000_000A;

        let res0 = extract_bits(test_num, 63, 1);
        let res1 = extract_bits(test_num, 0, 12);

        assert_eq!(res0, 1);
        assert_eq!(res1, 10);

    }
}
