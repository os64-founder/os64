use core::arch::asm;

// #[cfg(all(target_arch = "x86", target_os = "interix"))] 
#[inline(always)] 
pub unsafe fn asm_out_byte(port: u16, data: u8) {
    asm!("outb %al, %dx", in("dx") port, in("al") data, options(att_syntax));
}

// #[cfg(all(target_arch = "x86", target_os = "interix"))] 
#[inline(always)] 
pub unsafe fn asm_in_byte(port: u16) -> u8 {
    let data: u8;
    asm!("inb %dx, %al", out("al") data, in("dx") port, options(att_syntax));
    data  
}

// #[cfg(all(target_arch = "x86", target_os = "interix"))] 
#[inline(always)] 
pub unsafe fn asm_clear_interrupt_flag() {
    asm!("cli");
}

// #[cfg(all(target_arch = "x86", target_os = "interix"))] 
#[inline(always)] 
pub unsafe fn asm_set_interrupt_flag() {
    asm!("sti");
}
