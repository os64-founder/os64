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

pub unsafe fn asm_in_u16(port : u16,buffer : *mut u16, size : usize){
    asm!(
        "cld",          // 清方向标志,用于字符串操作
        "rep insw",     // 重复输入 nr 个字到 es:di
        "mfence",       //内存屏障,等待所有之前的内存访问完成
        in("dx") port,  // port 寄存器
        in("di") buffer,// es:di,目标缓冲区
        in("cx") size,  // cx,重复次数     
        options(nostack, preserves_flags)
    );
}

pub unsafe fn asm_in_u32(port : u16, buffer : *mut u32, size : usize){
    asm!(
        "cld",          // 清方向标志,用于字符串操作
        "rep insd",     // 重复输入 nr 个字到 es:di
        "mfence",       //内存屏障,等待所有之前的内存访问完成
        in("dx") port,  // port 寄存器
        in("di") buffer,// es:di,目标缓冲区
        in("cx") size,  // cx,重复次数     
        options(nostack, preserves_flags)
    );
}

pub unsafe fn asm_out_u32(port : u16, buffer : *const u32, size : usize){
    asm!(
        "cld",          // 清方向标志,用于字符串操作
        "rep outsd",    // 重复输出 nr 个字从 es:si
        "mfence",       //内存屏障,等待所有之前的内存访问完成
        in("dx") port,   // port 寄存器
        in("si") buffer, // es:si,源缓冲区
        in("cx") size,   // cx,重复次数
        options(nostack, preserves_flags)
    );
}
