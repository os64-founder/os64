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

// #[cfg(all(target_arch = "x86", target_os = "interix"))] 
#[inline(always)] 
pub fn asm_nop() {
    unsafe { asm!("nop"); }
}

pub unsafe fn asm_insw(port : u16,buffer : *mut u16, nr : usize){
    asm!(
        "cld",        // 清方向标志,用于字符串操作
        "rep insw",   // 重复输入 nr 个字到 es:di
        "mfence",   //内存屏障,等待所有之前的内存访问完成
        in("dx") port, // port 寄存器
        in("di") buffer, // es:di,目标缓冲区
        in("cx") nr,   // cx,重复次数     
        // out("memory") _, // 避免破坏任何寄存器
        options(nostack, preserves_flags)
    );
}
