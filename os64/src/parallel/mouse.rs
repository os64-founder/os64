
use lazy_static::lazy_static;
use crate::{architecture::x86_64_asm::{asm_out_byte, asm_in_byte, asm_nop}, serial_println};
use super::{apic::*, ring_buffer::{RingBuffer, RING_BUFFER_SIZE}};

#[repr(packed)]
struct mouse_packet {
    ///7:Y overflow,6:X overflow,5:Y sign bit,4:X sign bit,3:Always,2:Middle Btn,1:Right Btn,0:Left Btn
	flags : u8,
    ///X movement
	dx : i8,
    ///Y movement
	dy : i8,
}

const KBCMD_SENDTO_MOUSE    : u8 = 0xD4;
const MOUSE_ENABLE          : u8 = 0xF4;
const KBCMD_EN_MOUSE_INTFACE: u8 = 0xA8;

const PORT_MOUSE_DATA          : u16 = 0x60;
const PORT_MOUSE_COMMAND       : u16 = 0x64;

const KBSTATUS_IBF          : u8 = 0x02;
const KBSTATUS_OBF          : u8 = 0x01;

const SET_STATUS            : u8 = 0x60;
const GET_STATUS            : u8 = 0x20;


const KB_INIT_MODE          : u8 = 0x47;

pub static mut mouse_buffer : RingBuffer<u8> = RingBuffer {
    count: 0, 
    head: 0,
    tail: 0, 
    buffer: [0; RING_BUFFER_SIZE as usize],
};

static mut mouse_data : mouse_packet = mouse_packet{flags: 0,dx: 0, dy: 0};

static mut mouse_count : isize = 0;

pub fn mouse_init() {
    unsafe {
        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_COMMAND,GET_STATUS);

        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x1) !=0x1 {}
        let status = asm_in_byte(PORT_MOUSE_DATA) | 0x2;
        serial_println!("mouse_init(): status = {}",status);

        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_COMMAND,SET_STATUS);

        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_DATA,status & 0xDF);

        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_COMMAND,KBCMD_SENDTO_MOUSE);
        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_DATA,0xF6);
        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x1) !=0x1 {}
        asm_in_byte(PORT_MOUSE_DATA);

        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_COMMAND,KBCMD_SENDTO_MOUSE);
        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x2) !=0 {}
        asm_out_byte(PORT_MOUSE_DATA,MOUSE_ENABLE);
        while (asm_in_byte(PORT_MOUSE_COMMAND) & 0x1) !=0x1 {}
        asm_in_byte(PORT_MOUSE_DATA);
    }
}

pub fn analysis_mousecode() {

    let x = get_mousecode();

    unsafe{
        match (mouse_count) {
            0 => mouse_count+=1,
            1 => {
                mouse_data.flags = x;
                mouse_count+=1;
            },
            2 => {
                mouse_data.dx = x as i8;
                mouse_count+=1;
            },
            3 => {
                mouse_data.dy = x as i8;
                mouse_count = 1;
                serial_println!("(M:{},X:{},Y:{})\n",mouse_data.flags,mouse_data.dx,mouse_data.dy);
            },
            _ => {}
        }
    }
}

pub fn mouse_handler() {
    unsafe { 
        let x = asm_in_byte(PORT_MOUSE_DATA);
        mouse_buffer.push(x);
    }
}

pub fn get_mousecode() -> u8 {
    unsafe { 
        while mouse_buffer.is_empty() {
            asm_nop();
        }
        match mouse_buffer.pop() {
            None => 0,
            Some(p) => p,
        }
    }
}
