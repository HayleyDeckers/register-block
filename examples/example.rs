use register_block::register_block;

#[register_block]
pub struct TestRegs {
    #[register(offset = 0x00, access = "RW")]
    reg0: u32,
    #[register(offset = 0x04, access = "RO")]
    reg1: u32,
    #[register(offset = 0x08, access = "WO")]
    reg2: u32,
    // This should be allowed: RO overlaps with WO
    #[register(offset = 0x08, access = "RO")]
    reg2_ro: u32,
    #[register(offset = 0x0C, access = "WC")]
    reg3: u32,
    // This should cause a compile error: RW overlaps with RW
    // #[register(offset = 0x00, access = "RW")]
    // reg0_dup: u32,
    // This should cause a compile error: WO overlaps with RW
    // #[register(offset = 0x00, access = "WO")]
    // reg0_wo: u32,
}

fn main() {
    let buffer = [0u8; 0x10];
    let regs = TestRegs::new(&buffer[0] as *const u8 as usize);
    // The following methods should exist:
    let _ = regs.reg0().read();
    let _ = regs.reg1().read();
    regs.reg0().write(42);
    regs.reg2().write(1);
    regs.reg3().clear();
    let _ = regs.reg2_ro().read();
}
