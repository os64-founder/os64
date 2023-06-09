//                      ____________                          ____________
// Real Time Clock --> |            |   Timer -------------> |            |
// ACPI -------------> |            |   Keyboard-----------> |            |      _____
// Available --------> | Secondary  |----------------------> | Primary    |     |     |
// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
// Primary ATA ------> |            |   Floppy disk -------> |            |
// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
//

use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame,PageFaultErrorCode}};
use crate::{hlt_loop, parallel::mouse::{self, on_mouse_action}, device::disk::ide::ide_handler};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::{self, Mutex};

use crate::{serial_println, serial_print};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::global_descriptor_table::DOUBLE_FAULT_IST_INDEX); // new
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler); // new
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
        idt[InterruptIndex::Serial0.as_usize()].set_handler_fn(serial0_interrupt_handler);
        idt[InterruptIndex::Serial1.as_usize()].set_handler_fn(serial1_interrupt_handler);
        idt[InterruptIndex::IDE0.as_usize()].set_handler_fn(ide0_interrupt_handler);
        idt[InterruptIndex::IDE1.as_usize()].set_handler_fn(ide1_interrupt_handler);
        idt[InterruptIndex::SystemCall.as_usize()].set_handler_fn(system_call_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        mouse::init(on_mouse_action);
        idt
    };
}

pub fn init_interrupt_descriptor_table() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
    Serial0 = PIC_1_OFFSET + 3, 
    Serial1 = PIC_1_OFFSET + 4,
    Floppy = PIC_1_OFFSET + 6,
    Parallel = PIC_1_OFFSET + 7, 
    Mouse = PIC_1_OFFSET + 12,
    IDE0 = PIC_1_OFFSET + 14,
    IDE1 = PIC_1_OFFSET + 15, 
    SystemCall = 0x80, 
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => serial_print!("{}", character),
                DecodedKey::RawKey(key) => serial_print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}


extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}

lazy_static! {
    static ref MOUSE: Mutex<u8> =
        Mutex::new(0);
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    let mut mouse = MOUSE.lock();
    // serial_print!("-");
    mouse::mouse_handler();    

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

extern "x86-interrupt" fn ide0_interrupt_handler(_stack_frame: InterruptStackFrame) {
    ide_handler(0);
	unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::IDE0.as_u8());
    }
}

extern "x86-interrupt" fn ide1_interrupt_handler(_stack_frame: InterruptStackFrame) {
    ide_handler(1);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::IDE1.as_u8());
    }
}

extern "x86-interrupt" fn serial0_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!("---serial0---");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Serial0.as_u8());
    }
}

extern "x86-interrupt" fn serial1_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!("---serial1---");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Serial1.as_u8());
    }
}

extern "x86-interrupt" fn system_call_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!("---system_call---");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Serial1.as_u8());
    }
}
