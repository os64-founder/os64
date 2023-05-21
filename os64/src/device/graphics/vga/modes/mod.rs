pub mod graphics_320x200x256;
pub mod graphics_320x240x256;
pub mod graphics_640x480x16;

pub use graphics_320x200x256::Graphics320x200x256;
pub use graphics_320x240x256::Graphics320x240x256;
pub use graphics_640x480x16::Graphics640x480x16;

use crate::device::graphics::{drawing::colors, Color};


/// A helper trait used to interact with various vga screens.
pub trait Screen {
    /// The width of the `Screen`.
    const WIDTH: usize;
    /// The height of the `Screen`.
    const HEIGHT: usize;
    /// The size (total area) of the `Screen`.
    const SIZE: usize;
}


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color4 {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Pink = 5,
    Brown = 6,
    Silver = 7,
    Grey = 8,
    DarkBlue = 9,
    DarkGreen = 10,
    DarkCyan = 11,
    DarkRed = 12,
    Purple = 13,
    DarkYellow = 14,
    White = 15,
}

const ALLCOLOR4 :[Color4 ; 16] = [
    Color4::Black ,
    Color4::Blue ,
    Color4::Green,
    Color4::Cyan ,
    Color4::Red ,
    Color4::Pink ,
    Color4::Brown ,
    Color4::Silver ,
    Color4::Grey ,
    Color4::DarkBlue ,
    Color4::DarkGreen ,
    Color4::DarkCyan ,
    Color4::DarkRed ,
    Color4::Purple ,
    Color4::DarkYellow ,
    Color4::White ,
];

pub const ALLCOLOR4COLOR : [Color ; 16] = [
    colors::BLACK ,
    colors::BLUE ,
    colors::GREEN,
    colors::CYAN ,
    colors::RED ,
    colors::PINK ,
    colors::YELLOW ,
    colors::SILVER ,
    colors::GREY,
    colors::DARKBLUE ,
    colors::DARKGREEN ,
    colors::DARKCYAN ,
    colors::DARKRED ,
    colors::PURPLE ,
    colors::DARKYELLOW ,
    colors::WHITE ,
];
    
impl From<Color4> for u8 {
    fn from(color: Color4) -> u8 {
        color as u8
    }
}

impl From<Color> for Color4 {
    fn from(color: Color) -> Color4 {
        match color {
            colors::SILVER => Color4::Silver,
            colors::GREY => Color4::Grey,
            _ =>
            {
                let mut value : u8 = 0;
                let b = color.blue();
                let g = color.green();
                let r = color.red();
                value |= ((b >> 7) & 0x01) as u8; 
                value |= ((g >> 6) & 0x02) as u8; 
                value |= ((r >> 5) & 0x04) as u8;

                match value {
                    0 => {
                        value |= ((b >> 6) & 0x01) as u8; 
                        value |= ((g >> 5) & 0x02) as u8; 
                        value |= ((r >> 4) & 0x04) as u8; 
                        if value != 0 {
                            value |= 0x08
                        }
                    }
                    7 => {value = 0x0F}
                    _ => {}
                }

                ALLCOLOR4[value as usize]
            }
        }
    }
}

impl From<Color4> for Color {
    fn from(color: Color4) -> Color {
        ALLCOLOR4COLOR[color as usize]
    }
}

