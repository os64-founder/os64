use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{architecture::x86_64_asm::{asm_out_u8, asm_in_u8}, serial_println};

bitflags! { 
    /// Represents the flags currently set for the mouse.
    #[derive(Default)]
    pub struct MouseFlags: u8 {
        /// # [en]
        /// Whether or not the left mouse button is pressed.
        /// # [zh-CN]
        /// 已按下鼠标左键
        const LEFT_BUTTON = 0b0000_0001;
        
        /// # en
        /// Whether or not the right mouse button is pressed.
        /// # zh-CN
        /// 已按下鼠标右键
        const RIGHT_BUTTON = 0b0000_0010;

        /// # en
        /// Whether or not the middle mouse button is pressed.
        /// # zh-CN
        /// 已按下鼠标中键
        const MIDDLE_BUTTON = 0b0000_0100;

        /// # en
        /// Whether or not the packet is valid or not.
        /// # zh-CN
        /// 数据包是否有效?
        const ALWAYS_ONE = 0b0000_1000;

        /// # en
        /// Whether or not the x delta is negative.
        /// # zh-CN
        /// X值符号位
        const X_SIGN = 0b0001_0000;

        /// # en
        /// Whether or not the y delta is negative.
        /// # zh-CN
        /// Y值符号位
        const Y_SIGN = 0b0010_0000;

        /// # en
        /// Whether or not the x delta overflowed.
        /// # zh-CN
        /// X值溢出
        const X_OVERFLOW = 0b0100_0000;

        /// # en
        /// Whether or not the y delta overflowed.
        /// # zh-CN
        /// Y值溢出
        const Y_OVERFLOW = 0b1000_0000;
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct MouseAction {
    ///Mouse Flags
	flags : MouseFlags,
    ///X movement
	dx : i16,
    ///Y movement
	dy : i16,
}

impl  MouseAction {
    /// Returns a new `MouseAction`.
    pub const fn new() -> MouseAction {
        MouseAction {
            flags: MouseFlags::empty(),
            dx: 0,
            dy: 0,
        }
    }

    /// Returns true if the left mouse button is currently down.
    pub fn left_button_down(&self) -> bool {
        self.flags.contains(MouseFlags::LEFT_BUTTON)
    }

    /// Returns true if the left mouse button is currently up.
    pub fn left_button_up(&self) -> bool {
        !self.flags.contains(MouseFlags::LEFT_BUTTON)
    }

    /// Returns true if the right mouse button is currently down.
    pub fn right_button_down(&self) -> bool {
        self.flags.contains(MouseFlags::RIGHT_BUTTON)
    }

    /// Returns true if the right mouse button is currently up.
    pub fn right_button_up(&self) -> bool {
        !self.flags.contains(MouseFlags::RIGHT_BUTTON)
    }

    /// Returns true if the x axis has moved.
    pub fn x_moved(&self) -> bool {
        self.dx != 0
    }

    /// Returns true if the y axis has moved.
    pub fn y_moved(&self) -> bool {
        self.dy != 0
    }

    /// Returns true if the x or y axis has moved.
    pub fn moved(&self) -> bool {
        self.x_moved() || self.y_moved()
    }

    /// Returns the x delta of the mouse state.
    pub fn get_dx(&self) -> i16 {
        self.dx
    }

    /// Returns the y delta of the mouse state.
    pub fn get_dy(&self) -> i16 {
        self.dy
    }
}

const PORT_MOUSE_DATA           : u16 = 0x60;
const PORT_MOUSE_COMMAND        : u16 = 0x64;

const STATUS_INPUT_BUFFER_FULL  : u8 = 0x02;
const STATUS_OUTPUT_BUFFER_FULL : u8 = 0x01;

const COMMAND_SET_STATUS        : u8 = 0x60;
const COMMAND_GET_STATUS        : u8 = 0x20;
const COMMAND_SEND_TO_MOUSE     : u8 = 0xD4;
const SET_ENABLE                : u8 = 0xF4;
const SET_DEFAULTS              : u8 = 0xF6;

#[inline(always)]
pub unsafe fn wait_for_read() -> Result<(), &'static str> {
    let timeout = 100_000;
    for _ in 0..timeout {
        let value = asm_in_u8(PORT_MOUSE_COMMAND);
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
        let value = asm_in_u8(PORT_MOUSE_COMMAND);
        if (value & STATUS_INPUT_BUFFER_FULL) == 0 {
            return Ok(());
        }
    }
    Err("wait for mouse write timeout")
}

#[inline(always)]
pub unsafe fn write_command(data: u8) -> Result<(), &'static str> {
    wait_for_write()?;
    asm_out_u8(PORT_MOUSE_COMMAND,data);
    Ok(())
}

#[inline(always)]
pub unsafe fn read_data() -> Result<u8, &'static str> {
    wait_for_read()?;
    Ok(asm_in_u8(PORT_MOUSE_DATA))
}

#[inline(always)]
pub unsafe fn write_data(data: u8) -> Result<(), &'static str> {
    wait_for_write()?;
    asm_out_u8(PORT_MOUSE_DATA, data);
    Ok(())
}

#[inline(always)]
pub unsafe fn get_status() -> Result<u8, &'static str> {
    write_command(COMMAND_GET_STATUS)?;
    read_data()
}

#[inline(always)]
pub unsafe fn set_status(data : u8) -> Result<(), &'static str> {
    write_command(COMMAND_SET_STATUS)?;
    write_data(data)
}

#[inline(always)]
pub unsafe fn send_to_mouse(data : u8) -> Result<u8, &'static str> {
    write_command(COMMAND_SEND_TO_MOUSE)?;
    write_data(data)?;
    read_data()
}

lazy_static! {
    static ref MOUSE : Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub fn mouse_handler() {
    unsafe { 
        let x = asm_in_u8(PORT_MOUSE_DATA);
        // mouse_buffer.push(x);
        MOUSE.lock().process_packet(x);
    }
}

/// Attempts to initialize a `Mouse`. If successful, interrupts will be generated
pub fn init(on_mouse_action: fn(MouseAction)) -> Result<Mouse, &'static str> {
    {MOUSE.lock().on_action = Some(on_mouse_action);}
    unsafe {
        set_status((get_status()? | 0x02) & 0xDF)?;
        let result = send_to_mouse(SET_DEFAULTS)?;
        serial_println!("result = {:02x}",result);
        let result = send_to_mouse(SET_ENABLE)?;
        serial_println!("result = {:02x}",result);
    }
    Ok(Mouse::new())
}

pub fn on_mouse_action( mouse_action : MouseAction) {
    serial_println!("X:{},Y:{}\n",mouse_action.get_dx(),mouse_action.get_dy());
}

#[derive(Debug)]
pub struct Mouse {
    current_packet: u8,
    current_action: MouseAction,
    completed_action: MouseAction,
    pub on_action: Option<fn(MouseAction)>,
}

impl Default for Mouse {
    fn default() -> Mouse {
        Mouse::new()
    }
}

impl Mouse {
    /// Creates a new `Mouse`.
    pub const fn new() -> Mouse {
        Mouse {
            current_packet: 0,
            current_action: MouseAction::new(),
            completed_action: MouseAction::new(),
            on_action: None,
        }
    }

    /// Returns the last completed action of the mouse.
    pub fn get_action(&self) -> MouseAction {
        self.completed_action
    }

    /// Attempts to process a packet.
    pub fn process_packet(&mut self, packet: u8) {
        match self.current_packet {
            0 => {
                let flags = MouseFlags::from_bits_truncate(packet);
                if !flags.contains(MouseFlags::ALWAYS_ONE) {
                    return;
                }
                self.current_action.flags = flags;
            }
            1 => self.process_x_movement(packet),
            2 => {
                self.process_y_movement(packet);
                self.completed_action = self.current_action;
                // serial_println!("(M:{:?},X:{},Y:{})\n",self.completed_action.flags,self.completed_action.dx,self.completed_action.dy);
                if let Some(on_action) = self.on_action {
                    on_action(self.completed_action);
                }
            }
            _ => unreachable!(),
        }
        self.current_packet = (self.current_packet + 1) % 3;
    }

    /// Sets the `on_complete` function to be called when a packet is completed.
    pub fn set_on_action(&mut self, handler: fn(MouseAction)) {
        self.on_action = Some(handler);
    }

    fn process_x_movement(&mut self, packet: u8) {
        if !self.current_action.flags.contains(MouseFlags::X_OVERFLOW) {
            self.current_action.dx = if self.current_action.flags.contains(MouseFlags::X_SIGN) {
                self.sign_extend(packet)
            } else {
                packet as i16
            };
        }
    }

    fn process_y_movement(&mut self, packet: u8) {
        if !self.current_action.flags.contains(MouseFlags::Y_OVERFLOW) {
            self.current_action.dy = if self.current_action.flags.contains(MouseFlags::Y_SIGN) {
                self.sign_extend(packet)
            } else {
                packet as i16
            };
        }
    }

    fn sign_extend(&self, packet: u8) -> i16 {
        ((packet as u16) | 0xFF00) as i16
    }
}