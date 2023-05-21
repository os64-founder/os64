use crate::{device::{graphics::{vga::{VGA, VideoMode, registers::PlaneMask}, GraphicsDriver, Size, Color, Color8}}};
use super::Screen;
// use font8x8::UnicodeFonts;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const SIZE: usize = (WIDTH * HEIGHT) / 4;

#[derive(Debug, Clone, Copy, Default)]
pub struct Graphics320x240x256;

impl Screen for Graphics320x240x256 {
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const SIZE: usize = SIZE;
}

impl GraphicsDriver for Graphics320x240x256 {
    fn init(&mut self) {
        let mut vga = VGA.lock();
        vga.set_video_mode(VideoMode::Mode320x240x256);
        // vga.color_palette_registers.load_palette(&DEFAULT_PALETTE_COLOR16);
    }

    fn get_full_screen_size(&self) -> Size<usize> {
        Size{w:WIDTH,h:HEIGHT}
    }

    fn clear_screen(&mut self, color: Color) {
        let frame_buffer = self.get_frame_buffer();
        VGA.lock()
            .sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);
        unsafe {
            frame_buffer.write_bytes(Color8::from(color).value, Self::SIZE);
        }
    }

    fn get_pixel(&self, x : usize, y : usize) -> Option<Color> {
        Option::None
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> bool {
        if(x < WIDTH && y < HEIGHT) {
            let frame_buffer = self.get_frame_buffer();
            unsafe {
                let offset = (WIDTH * y + x) / 4;
                let plane_mask = 0x1 << (x & 3);
                VGA.lock()
                    .sequencer_registers
                    .set_plane_mask(PlaneMask::from_bits(plane_mask).unwrap());
                frame_buffer.add(offset).write_volatile(Color8::from(color).value);
            }
            true
            } else {
            false
        }
    }

    // fn draw_character(&self, x: usize, y: usize, character: char, color: Color) {
    //     let character = match font8x8::BASIC_FONTS.get(character) {
    //         Some(character) => character,
    //         // Default to a filled block if the character isn't found
    //         None => font8x8::unicode::BLOCK_UNICODE[8].byte_array(),
    //     };

    //     for (row, byte) in character.iter().enumerate() {
    //         for bit in 0..8 {
    //             match *byte & 1 << bit {
    //                 0 => (),
    //                 _ => self.set_pixel(x + bit, y + row, color),
    //             }
    //         }
    //     }
    // }
}

impl Graphics320x240x256 {
    fn get_frame_buffer(&self) -> *mut u8 {
        usize::from(VGA.lock().get_frame_buffer()) as *mut u8
    }

    /// Creates a new `Graphics320x240x256`.
    pub const fn new() -> Graphics320x240x256 {
        Graphics320x240x256
    }
}
