
use lazy_static::lazy_static;
use crate::{architecture::x86_64_asm::{asm_out_byte, asm_in_byte, asm_nop}, serial_println};
use super::{ring_buffer::{RingBuffer, RING_BUFFER_SIZE}};

#[repr(packed)]
struct mouse_packet {
    ///7:Y overflow,6:X overflow,5:Y sign bit,4:X sign bit,3:Always,2:Middle Btn,1:Right Btn,0:Left Btn
	flags : u8,
    ///X movement
	dx : i8,
    ///Y movement
	dy : i8,
}

const PORT_MOUSE_DATA           : u16 = 0x60;
const PORT_MOUSE_COMMAND        : u16 = 0x64;

const STATUS_INPUT_BUFFER_FULL  : u8 = 0x02;
const STATUS_OUTPUT_BUFFER_FULL : u8 = 0x01;

const SET_STATUS                : u8 = 0x60;
const GET_STATUS                : u8 = 0x20;

const COMMAND_SENDTO_MOUSE      : u8 = 0xD4;
const SET_ENABLE                : u8 = 0xF4;
const SET_DEFAULTS              : u8 = 0xF6;

pub static mut mouse_buffer : RingBuffer<u8> = RingBuffer {
    count: 0, 
    head: 0,
    tail: 0, 
    buffer: [0; RING_BUFFER_SIZE as usize],
};

static mut mouse_data : mouse_packet = mouse_packet{flags: 0,dx: 0, dy: 0};
static mut mouse_count : isize = 0;

#[inline(always)]
pub unsafe fn wait_for_read() -> Result<(), &'static str> {
    let timeout = 100_000;
    for _ in 0..timeout {
        let value = asm_in_byte(PORT_MOUSE_COMMAND);
        if (value & STATUS_OUTPUT_BUFFER_FULL) != 0 {
            return Ok(());
        }
    }
    Err("wait for mouse read timeout")
}

#[inline(always)]
pub unsafe fn wait_for_write() -> Result<(), &'static str> {
    let timeout = 100_000;
    for _ in 0..timeout {
        let value = asm_in_byte(PORT_MOUSE_COMMAND);
        if (value & STATUS_INPUT_BUFFER_FULL) == 0 {
            return Ok(());
        }
    }
    Err("wait for mouse write timeout")
}

#[inline(always)]
pub unsafe fn write_command(data: u8) -> Result<(), &'static str> {
    wait_for_write()?;
    asm_out_byte(PORT_MOUSE_COMMAND,data);
    Ok(())
}

#[inline(always)]
pub unsafe fn read_data() -> Result<u8, &'static str> {
    wait_for_read()?;
    Ok(asm_in_byte(PORT_MOUSE_DATA))
}

#[inline(always)]
pub unsafe fn write_data(data: u8) -> Result<(), &'static str> {
    wait_for_write()?;
    asm_out_byte(PORT_MOUSE_DATA, data);
    Ok(())
}

#[inline(always)]
pub unsafe fn get_status() -> Result<u8, &'static str> {
    write_command(GET_STATUS)?;
    read_data()
}

#[inline(always)]
pub unsafe fn set_status(data : u8) -> Result<(), &'static str> {
    write_command(SET_STATUS)?;
    write_data(data)
}

#[inline(always)]
pub unsafe fn send_to_mouse(data : u8) -> Result<u8, &'static str> {
    write_command(COMMAND_SENDTO_MOUSE)?;
    write_data(data)?;
    read_data()
}

pub fn mouse_init() -> Result<(), &'static str> {
    unsafe {
        set_status((get_status()? | 0x02) & 0xDF)?;
        send_to_mouse(SET_DEFAULTS)?;
        send_to_mouse(SET_ENABLE)?;
    }
    Ok(())
}

pub fn analysis_mousecode() {

    let x = get_mousecode();

    unsafe{
        match (mouse_count) {
            0 => {
                mouse_data.flags = x;
                mouse_count+=1;
            },
            1 => {
                mouse_data.dx = x as i8;
                mouse_count+=1;
            },
            2 => {
                mouse_data.dy = x as i8;
                mouse_count = 0;
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
