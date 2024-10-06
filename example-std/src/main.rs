extern crate m6binpack;

pub use m6binpack::unpack;

fn main() {
    println!("Hello, world!");

    let cause: usize = 0x8000_0000_0000_000A;

    unpack! {
        <cause_num: usize: 63><is_async: bool: 1> = cause;
        <B0: usize: 12><B1: u8: 4><B2: u8: 8> = cause;
    };

    println!("cause_num: {}, is_async: {}", cause_num, is_async);
    println!("B0: {}, B1: {}, B2: {}", B0, B1, B2);
}
