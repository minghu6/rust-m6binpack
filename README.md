
## Usage

### `extract_bits`

提供一种灵活的解包 bit 数据的方法，更正式地比如 [bitflags](https://github.com/bitflags/bitflags) 需要定义专门的结构体。

```rust
let test_num = 0x8000_0000_0000_000A;

let res0 = extract_bits(test_num, 63, 1);
let res1 = extract_bits(test_num, 0, 12);

assert_eq!(res0, 1);
assert_eq!(res1, 10);
```

### `unpack`

```rust
let cause: usize = 0x8000_0000_0000_000A;

unpack! {
    <cause_num: usize: 63><is_async: bool: 1> = cause;
    <B0: usize: 12><B1: u8: 4><B2: u8: 8> = cause;
};

println!("cause_num: {}, is_async: {}", cause_num, is_async);
println!("B0: {}, B1: {}, B2: {}", B0, B1, B2);
```
