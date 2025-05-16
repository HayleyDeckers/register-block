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
    #[register(offset = 0x0C, access = "Clear")]
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
    let _ = regs.read_reg0();
    let _ = regs.read_reg1();
    regs.write_reg0(42);
    regs.write_reg2(1);
    regs.clear_reg3();
    let _ = regs.read_reg2_ro();
}
