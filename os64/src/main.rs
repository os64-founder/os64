//---------------------------------------------------------
// OS64 Source File
//  
//  src/main.rs
// 
//  author: TonyLiu
// 
//---------------------------------------------------------
#![no_std]
#![no_main]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(core_intrinsics)]
#![test_runner(os64::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

// use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
// use os64::parallel::{executor::Executor, Task, keyboard};
use bootloader::{BootInfo, entry_point, bootinfo};
use x86_64::VirtAddr;
use os64::{device::{serial::_print, graphics::{GraphicsDriver, drawing::{canvas::{ScreenCanvas, Canvas}, windows::{widget_base::{add_child, Widget}, win31_style::{create_cursor_widget, create_window, BorderKind, create_desktop}}, colors}, vga::modes::{Graphics640x480x16, ALLCOLOR4COLOR}, Rect, Point, Size}, devices_init}, memory::{self, BootInfoFrameAllocator}, parallel::{mouse::{self}, process::Process}};

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        _print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_print!("{}", info);
    crate::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    os64::test_panic_handler(info)
}

entry_point!(kernel_main);

//#[no_mangle]
//pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial_println!("OS64 start ...");

    serial_println!("offset = 0x{:016x}", boot_info.physical_memory_offset);
    // serial_println!("memory_map = {:?}", boot_info.memory_map);

    os64::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    os64::memory::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    os64::device::clock::real_time_clock::get_datetime();
    devices_init();
    vga_test();

    let process = Process::read("test",boot_info.physical_memory_offset);

    // unsafe{ 
    //     asm!("int 0x80");
    // }

    // task_test();
    // print_l4_table(boot_info);
    // test_translate(boot_info);
    // test_translate_v2(boot_info);
    // test_new(boot_info);
    // let x = Box::new(41);
    // test_alloc();

    // as before
    #[cfg(test)]
    test_main();

    serial_println!("It did not crash!");

    loop {
        x86_64::instructions::hlt();
        // os64::hlt_loop();
    }

}

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

fn vga_test() {
    let mut vga = Graphics640x480x16::new();
    // let mut vga = graphics_320x200x256::Graphics320x200x256::new();
    // let mut vga = graphics_320x240x256::Graphics320x240x256::new();
    GraphicsDriver::init(&mut vga);
    GraphicsDriver::clear_screen(&mut vga, colors::SILVER);

    let mut canvas = ScreenCanvas::from_driver(&mut vga);
    let screen_rect = canvas.rect;
    
    let desktop = create_desktop(screen_rect);
    
    let rect = Rect { left_top: Point { x: 160, y: 100 }, size: Size { w:320, h: 240 } };
    let welcome_window = create_window("Welcome to OS64", rect, BorderKind::Resizable);

    add_child(&desktop, welcome_window);

    let center = Point{ x : screen_rect.width() as isize / 2, y : screen_rect.height() as isize / 2 };
    let mut cursor = create_cursor_widget(center);
    add_child(&desktop, cursor);

    desktop.repaint(&mut canvas, screen_rect.left_top, screen_rect);

    // canvas.draw_text(0, 0, "Hello World!", colors::BLACK, colors::TRANSPARENT);
    // for i in 0..20 {
    //     canvas.set_pixel(i,i,colors::RED);
    // }

    canvas.draw_x_line(0, 447, 640, colors::BLACK);
    for i in 0..16 {
        canvas.fill_rectangle(i*40, 448, 40, 16, ALLCOLOR4COLOR[(i as usize + 8) % 16]);
        canvas.fill_rectangle(i*40, 464, 40, 16, ALLCOLOR4COLOR[i as usize]);
    }
}

// fn test_alloc() {
//     // allocate a number on the heap
//     let heap_value = Box::new(41);
//     serial_println!("heap_value at {:p}", heap_value);

//     // create a dynamically sized vector
//     let mut vec = Vec::new();
//     for i in 0..500 {
//         vec.push(i);
//     }
//     serial_println!("vec at {:p}", vec.as_slice());

//     // create a reference counted vector -> will be freed when count reaches 0
//     let reference_counted = Rc::new(vec![1,2,3]);
//     //let reference_counted = Rc::new(vec![1, 2, 3]);
//     let cloned_reference = reference_counted.clone();
//     serial_println!("current reference count is {}", Rc::strong_count(&cloned_reference));
//     core::mem::drop(reference_counted);
//     serial_println!("reference count is {} now", Rc::strong_count(&cloned_reference));
// }

// fn test_new(boot_info: &'static BootInfo) {
//     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
//     let mut mapper = unsafe { memory::init(phys_mem_offset) };
//     // let mut frame_allocator = memory::EmptyFrameAllocator;
//     let mut frame_allocator = unsafe {
//         BootInfoFrameAllocator::init(&boot_info.memory_map)
//     };

//     // 映射未使用的页
//     let page = Page::containing_address(VirtAddr::new(0));
//     memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);


//     // 通过新的映射将字符串 `New!`  写到屏幕上。
//     let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
//     unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
// }

// fn test_translate_v2(boot_info: &'static BootInfo) {
//     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

//     let addresses = [
//         // the identity-mapped vga buffer page
//         0xb8000,
//         // some code page
//         0x201008,
//         // some stack page
//         0x0100_0020_1a10,
//         //0x180_0000_0000
//         // virtual address mapped to physical address 0
//         boot_info.physical_memory_offset,
//     ];

//     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
//     // new: initialize a mapper
//     let mapper = unsafe { memory::init(phys_mem_offset) };

//     // VirtAddr(0xb8000) -> Some(PhysAddr(0xb8000))
//     // VirtAddr(0x201008) -> Some(PhysAddr(0x401008))
//     // VirtAddr(0x10000201a10) -> Some(PhysAddr(0x27fa10))
//     // VirtAddr(0x18000000000) -> Some(PhysAddr(0x0))
//     for &address in &addresses {
//         let virt = VirtAddr::new(address);
//         // new: use the `mapper.translate_addr` method
//         let phys = mapper.translate_addr(virt);
//         crate::serial_println!("{:?} -> {:?}", virt, phys);
//     }
// }

// fn test_translate(boot_info: &'static BootInfo) {
//     use x86_64::{structures::paging::PageTable, VirtAddr};
//     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

//     let addresses = [
//         // the identity-mapped vga buffer page
//         0xb8000,
//         // some code page
//         0x201008,
//         // some stack page
//         0x0100_0020_1a10,
//         //0x180_0000_0000
//         // virtual address mapped to physical address 0
//         boot_info.physical_memory_offset,
//     ];

//     // VirtAddr(0xb8000) -> Some(PhysAddr(0xb8000))
//     // VirtAddr(0x201008) -> Some(PhysAddr(0x401008))
//     // VirtAddr(0x10000201a10) -> Some(PhysAddr(0x27fa10))
//     // panicked at 'huge pages not supported', src/memory.rs:79:43
//     for &address in &addresses {
//         let virt = VirtAddr::new(address);
//         let phys = unsafe { translate_addr(virt, phys_mem_offset) };
//         crate::serial_println!("{:?} -> {:?}", virt, phys);
//     }
// }

// fn print_l4_table(boot_info: &'static BootInfo) {
//     use x86_64::{structures::paging::PageTable, VirtAddr};
//     //boot_info.physical_memory_offset
//     let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
//     let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

//     //虚拟机上的结果
//     for (i, entry) in l4_table.iter().enumerate() {
//         if !entry.is_unused() {
//             os64::serial_println!("L4 Entry {}: {:?}", i, entry);

//             // //　接着打印 L3 条目， 条目太多了
//             // // get the physical address from the entry and convert it
//             // let phys = entry.frame().unwrap().start_address();
//             // let virt = phys.as_u64() + boot_info.physical_memory_offset;
//             // let ptr = VirtAddr::new(virt).as_mut_ptr();
//             // let l3_table: &PageTable = unsafe { &*ptr };

//             // // print non-empty entries of the level 3 table
//             // for (i, entry) in l3_table.iter().enumerate() {
//             //     if !entry.is_unused() {
//             //         println!("  L3 Entry {}: {:?}", i, entry);
//             //     }
//             // }
//         }
//     }
// }


// async fn async_number() -> u32 {
//     42
// }

// async fn example_task() {
//     let number = async_number().await;
//     serial_println!("async number: {}", number);
// }

// fn task_test(){
//     let mut executor = Executor::new();
//     executor.spawn(Task::new(example_task()));
//     executor.spawn(Task::new(keyboard::print_keypresses()));
//     executor.run();
// }
