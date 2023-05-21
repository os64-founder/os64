use crate::device::graphics::{GraphicsDriver, force_between, Color, Size, Rect, Point};
use crate::device::graphics::algorithm::Bresenham;


use crate::{device::{graphics::vga::{VGA, VideoMode, registers::{graphics_controller, PlaneMask}}}};
use super::{Screen, Color4, ALLCOLOR4, ALLCOLOR4COLOR};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const SIZE: usize = (WIDTH * HEIGHT) / 8;
const WIDTH_IN_BYTES: usize = WIDTH / 8;

#[derive(Debug, Clone, Copy, Default)]
pub struct Graphics640x480x16;

impl Screen for Graphics640x480x16 {
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const SIZE: usize = SIZE;
}

impl GraphicsDriver for Graphics640x480x16 {
    fn init(&mut self) {
        let mut vga = VGA.lock();
        vga.set_video_mode(VideoMode::Mode640x480x16);
        // vga.color_palette_registers.load_palette(&DEFAULT_PALETTE_COLOR16);
    }

    fn get_full_screen_size(&self) -> Size<usize> {
        Size{w:WIDTH,h:HEIGHT}
    }

    fn clear_screen(&mut self, color: Color) {
        self.set_write_mode_2();
        unsafe {
            self.get_frame_buffer()
                .write_bytes(Color4::from(color) as u8, Self::SIZE);
        }
    }

    fn fill_rectangle(&mut self, rect : Rect, color : Color) {
        let x1 = force_between(rect.left(),0,WIDTH as isize) as usize;
        let x2 = force_between(rect.right(),0,WIDTH as isize) as usize;
        let y1 = force_between(rect.top(),0,HEIGHT as isize) as usize;
        let y2 = force_between(rect.bottom(),0,HEIGHT as isize) as usize;
        let color = Color4::from(color) as u8;
        let frame_buffer = self.get_frame_buffer();
        let mut l = VGA.lock();
        for ix in 0..8 {
            let pixel_mask = 0x80 >> ix;
            l.graphics_controller_registers.set_bit_mask(pixel_mask);
            for x in (x1..x2).into_iter().filter(|x| x % 8 == ix) {
                for y in y1..y2 {
                    let offset = x / 8 + y * WIDTH_IN_BYTES;
                    unsafe {
                        frame_buffer.add(offset).read_volatile();
                        frame_buffer.add(offset).write_volatile(color);
                    }
                }
            }
        }
    }

    // fn draw_character(&self, x: usize, y: usize, character: char, color: Color) {
    //     self.set_write_mode_2();
    //     let character = match font8x8::BASIC_FONTS.get(character) {
    //         Some(character) => character,
    //         // Default to a filled block if the character isn't found
    //         None => font8x8::unicode::BLOCK_UNICODE[8].byte_array(),
    //     };

    //     for (row, byte) in character.iter().enumerate() {
    //         for bit in 0..8 {
    //             match *byte & 1 << bit {
    //                 0 => (),
    //                 _ => self._set_pixel(x + bit, y + row, Color4::from(color)),
    //             }
    //         }
    //     }
    // }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> bool {
        if x < WIDTH && y < HEIGHT {
            self.set_write_mode_2();
            self._set_pixel(x, y, Color4::from(color) as u8);
            true
        } else {
            false
        }
    }

    fn get_pixel(&self, x : usize, y : usize) -> Option<Color> {
        Some(ALLCOLOR4COLOR[(self._get_pixel(x, y) & 0xF) as usize])
    }

    fn draw_line(&mut self, start : Point<isize>, end : Point<isize>, color : Color){
        self.set_write_mode_0(color);
        let color = Color4::from(color) as u8;
        for point in Bresenham::new(start, end) {
            if point.x >=0 && point.y >=0 {
                self._set_pixel(point.x as usize, point.y as usize, color);
            }
        }
    }
}

impl Graphics640x480x16 {
    fn get_frame_buffer(&self) -> *mut u8 {
        usize::from(VGA.lock().get_frame_buffer()) as *mut u8
    }

    pub const fn new() -> Graphics640x480x16 {
        Graphics640x480x16
    }

    fn set_write_mode_0(self, color: Color) {
        let mut vga = VGA.lock();
        vga.graphics_controller_registers.write_set_reset(Color4::from(color) as u8);
        vga.graphics_controller_registers
            .write_enable_set_reset(0xF);
        vga.graphics_controller_registers
            .set_write_mode(graphics_controller::WriteMode::Mode0);
    }

    fn set_write_mode_2(self) {
        let mut vga = VGA.lock();
        vga.graphics_controller_registers
            .set_write_mode(graphics_controller::WriteMode::Mode2);
        vga.graphics_controller_registers.set_bit_mask(0xFF);
        vga.sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);
    }

    #[inline]
    fn _set_pixel(self, x: usize, y: usize, color: u8) {
        let frame_buffer = self.get_frame_buffer();
        let offset = x / 8 + y * WIDTH_IN_BYTES;
        let pixel_mask = 0x80 >> (x & 0x07);
        VGA.lock()
            .graphics_controller_registers
            .set_bit_mask(pixel_mask);
        unsafe {
            frame_buffer.add(offset).read_volatile();
            frame_buffer.add(offset).write_volatile(color);
        }
    }

    #[inline]
    fn _get_pixel(self, x: usize, y: usize) -> u8 {
        let frame_buffer = self.get_frame_buffer();
        let offset = x / 8 + y * WIDTH_IN_BYTES;
        let pixel_mask = 0x80 >> (x & 0x07);
        VGA.lock()
            .graphics_controller_registers
            .set_bit_mask(pixel_mask);
        unsafe {
            frame_buffer.add(offset).read_volatile()
        }
    }
}
