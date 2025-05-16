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

/// a MMIO register pointer that can be read
pub struct RO<T>(*const T);
impl<T> RO<T> {
    pub unsafe fn new(address: usize) -> Self {
        RO(address as *const T)
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
}

/// a MMIO register pointer that can be written to
pub struct WO<T>(*mut T);
impl<T> WO<T> {
    pub unsafe fn new(address: usize) -> Self {
        WO(address as *mut T)
    }
    pub fn write(&self, value: T) {
        unsafe { self.0.write_volatile(value) }
    }
}

/// a MMIO register pointer that can be read and written to
pub struct RW<T>(*mut T);
impl<T> RW<T> {
    pub unsafe fn new(address: usize) -> Self {
        RW(address as *mut T)
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }

    pub fn write(&self, value: T) {
        unsafe { self.0.write_volatile(value) }
    }

    pub fn modify<F>(&self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        self.write(f(self.read()));
    }
}

/// a MMIO register pointer that can be written to to clear the register
pub struct WC<T>(*mut T);
impl<T: Default> WC<T> {
    pub unsafe fn new(address: usize) -> Self {
        WC(address as *mut T)
    }
    pub fn clear(&self) {
        // todo: we don't really need to use Default here, but it's a good placeholder
        unsafe { self.0.write_volatile(T::default()) }
    }
}

/// a MMIO register pointer that can be read from and doing so will clear the register
pub struct RC<T>(*mut T);
impl<T> RC<T> {
    pub unsafe fn new(address: usize) -> Self {
        RC(address as *mut T)
    }
    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }
}
