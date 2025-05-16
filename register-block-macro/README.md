# register-block-macro

A procedural macro for generating safe and ergonomic register blocks and accessors for memory-mapped peripherals in Rust.

## Features
- **Enforces register offset safety:**
  - No two RW/WO/Clear fields may overlap
  - RO may only overlap with WO or Clear
  - Compile-time errors for invalid overlaps
- **Flexible access types:** `RW`, `RO`, `WO`, `Clear`

## Usage
1. Add this crate as a dependency to your project.
2. annotate your register block struct with `#[register_block]` and each field with `#[register(offset = ..., access = ...)]`.
   
## Example

```rust
use register_block_macro::register_block;

// Your trait for base address (must be in scope)
pub trait BaseAddress: Copy {
    fn base_address(self) -> usize;
}
impl BaseAddress for usize {
    fn base_address(self) -> usize { self }
}

#[register_block]
pub struct UART {
    /// Data Register
    #[register(offset = 0x00, access = "RW")]
    dr: u32,
    /// Status Register (read-only)
    #[register(offset = 0x04, access = "RO")]
    sr: u32,
    /// Error Clear Register (write-only)
    #[register(offset = 0x08, access = "WO")]
    ecr: u32,
    /// Interrupt Clear Register (clear)
    #[register(offset = 0x0C, access = "Clear")]
    icr: u32,
    /// Status Register (read-only, allowed to overlap with WO/Clear)
    #[register(offset = 0x08, access = "RO")]
    sr_ro: u32,
}

fn main() {
    let uart = UART { base: 0x4000_0000usize };
    let _ = uart.read_dr();
    let _ = uart.read_sr();
    uart.write_dr(123);
    uart.write_ecr(0);
    uart.clear_icr();
    let _ = uart.read_sr_ro();
}
```

