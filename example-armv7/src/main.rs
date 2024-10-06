#![allow(non_snake_case)]
#![allow(unused_variables)]

#![no_std]
#![no_main]

use core::mem::size_of;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

pub use m6binpack:: unpack;

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();
    hprintln!("pointer width: {}", size_of::<usize>() * 8).unwrap();

    #[cfg(target_pointer_width = "64")]
    let cause: usize = 0x8000_0000_0000_000A;

    #[cfg(target_pointer_width = "32")]
    let cause: usize = 0x8000_000A;

    unpack!{
        <cause_num: usize: 31><is_async: bool: 1> = cause;
        <B0: usize: 12><B1: u8: 4><B2: u8: 8> = cause;
    }

    hprintln!("cause_num: {}, is_async: {}", cause_num, is_async).unwrap();
    hprintln!("B0: {}, B1: {}, B2: {}", B0, B1, B2).unwrap();

    // hprintln!("Press C-X A to exit.").unwrap();
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    debug::exit(debug::EXIT_SUCCESS);

    loop {
        // your code goes here
    }
}
