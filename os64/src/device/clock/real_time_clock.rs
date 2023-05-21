use crate::architecture::x86_64_asm::{asm_out_byte, asm_in_byte, asm_clear_interrupt_flag, asm_set_interrupt_flag};


pub(crate) const NAME: &'static str = "/Device/RealTimeClock";

pub unsafe fn cmos_read(addr : u8) -> u8 {
    asm_out_byte(0x70, addr);
    let v = asm_in_byte(0x71);
    (((v & 0xF0) >> 4) * 10 )+ (v & 0x0F)
}

pub fn get_datetime() {
    let mut year= 0;
    let mut year_head = 0;
    let mut month = 0;
    let mut day = 0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    unsafe {
        asm_clear_interrupt_flag();
        year = cmos_read(0x09) as u32;
        year_head = cmos_read(0x32) as u32;
        month = cmos_read(0x08);
        day = cmos_read(0x07);
        hour = cmos_read(0x04);
        minute = cmos_read(0x02);
        second = cmos_read(0x00);
        asm_set_interrupt_flag();
    }
    // crate::serial_println!("Current value is {}-{}-{}T{}:{}:{}", year_head * 100 + year,  month, day, hour, minute, second);
}
