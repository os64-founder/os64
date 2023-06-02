#![no_std]
#![no_main]

use core::arch::asm;

//最早实现的 5 个系统调用
const OS64_API_EXIT             : u64 = 0x00000001;
const OS64_API_YIELD            : u64 = 0x00000002;
const OS64_API_PRINT            : u64 = 0x00000003;
const OS64_API_HEAP_ALLOC       : u64 = 0x00000004;
const OS64_API_HEAP_FREE        : u64 = 0x00000005;

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[no_mangle] 
pub extern "C" fn eh_personality() {}

pub fn os64_api_call(api_index : u64) {
    unsafe {
        asm!(
            "syscall",// execute system call
            in("ax") api_index,
        );
    }
}

// pub fn os64_api_call_6(
//     api_index : u64,
//     arg0 : u64,
//     arg1 : u64,
//     arg2 : u64,
//     arg3 : u64,
//     arg4 : u64,
//     arg5 : u64,
//     ) -> u64 
// {
//     let mut result: u64;
//     unsafe {
//         asm!(
//             "syscall            ",  // execute system call
//             // "mov {}, rax        ",   // return value to result
//             in("rax") api_index,
//             in("rbx") arg0, in("rcx") arg1,
//             in("rdx") arg2, in("r8") arg3,
//             in("r9")  arg4, in("r10") arg5,  
//             out("rax") result,
//         );
//     }
//     result
// }

pub fn os64_api_exit(ret : u64) {
    os64_api_call(OS64_API_EXIT);
}

pub fn os64_api_yield(ret : u64) {
    os64_api_call(OS64_API_YIELD);
}

#[no_mangle]
pub extern "C" fn _start() {
    // os64_api_yield(0);
    os64_api_exit(0);
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    hlt_loop();
}
