//! Re-exports the register_block macro and provides the BaseAddress trait and FixedAddress type.
pub use register_block_macro::register_block;

/// Trait for types that can provide a base address for a register block.
pub trait BaseAddress: Copy {
    fn base_address(self) -> usize;
}

impl BaseAddress for usize {
    fn base_address(self) -> usize {
        self
    }
}

/// Zero-sized type for compile-time constant base addresses.
#[derive(Debug, Clone, Copy)]
pub struct ConstantAddress<const BASE: usize>;
impl<const BASE: usize> BaseAddress for ConstantAddress<BASE> {
    fn base_address(self) -> usize {
        BASE
    }
}
