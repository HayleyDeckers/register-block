# Register Block

This crate provides a safe, ergonomic, way to define memory-mapped register blocks in Rust.

## Usage Example

```rust
use register_block::register_block;

#[register_block]
pub struct UART {
    #[register(offset = 0x00, access = "RW")]
    dr: u32,
    #[register(offset = 0x04, access = "RO")]
    sr: u32,
    #[register(offset = 0x08, access = "WO")]
    ecr: u32,
    #[register(offset = 0x0C, access = "Clear")]
    icr: u32,
}

fn main() {
    let uart = UART::new(0x4000_0000usize);
    // or 
    // let uart = UART::new(register_block::ConstantAddress<0x4000_000>);
    let _ = uart.read_dr();
    uart.write_dr(123);
    uart.clear_icr();
}
```

See the macro and trait documentation for more details.
