use core::cmp;


use crate::device::graphics::{Size, Color, Rect, Point, algorithm::Bresenham};

use super::{colors, ascii_font};

// enum CanvasDevice {
//     Screen,
//     Memory
// }

fn force_between<T : Ord>(value : T, min : T, max : T) -> T {
    cmp::max( cmp::min( value, max ), min)
}

pub trait Canvas {    
    fn size(&self) -> Size<usize>;
    fn get_pixel(&self, x : usize, y : usize) -> Option<Color>;
    fn set_pixel(&mut self, x : usize, y : usize, c : Color) -> bool;
    fn fill_rect(&mut self, rect : Rect, color : Color);

    fn width(&self) -> usize { self.size().w }
    fn height(&self) -> usize { self.size().h }

    fn force_to_between<T : Ord>(value : T, min : T, max : T) -> T {
        cmp::min(cmp::max(value, min) , max)
    }

    fn draw_x_line(&mut self, x : isize, y : isize, w : usize, color : Color) {
        if y >= 0 && (y as usize) < self.height() {
            let x_min = cmp::max(x, 0) as usize;
            let x_max = cmp::min(self.width(),x as usize + w);
            for ix in x_min..x_max  {
                self.set_pixel(ix,y as usize,color);
            }
        }
    }

    fn draw_y_line(&mut self, x : isize, y : isize, h : usize, color : Color) {
        if x >= 0 && (x as usize) < self.width() {
            let y_min = cmp::max(y, 0) as usize;
            let y_max = cmp::min(self.height(),y as usize + h);
            for iy in y_min..y_max  {
                self.set_pixel(x as usize,iy,color);
            }
        }
    }
    
    fn draw_rect(&mut self, r : Rect, color : Color) {
        self.draw_rectangle(r.left(), r.top(), r.width(), r.height(), color );
    }

    fn draw_rectangle(&mut self, x : isize, y : isize, w : usize, h : usize, color : Color) {
        self.draw_x_line(x, y, w, color);
        self.draw_x_line(x, y + (h as isize) - 1, w, color);
        self.draw_y_line(x, y, h, color);
        self.draw_y_line(x + (w as isize) - 1, y, h, color);
    }
    
    fn draw_rect_3d(&mut self, r : Rect, color_left_top : Color, color_right_bottom : Color) {
        self.draw_x_line(r.left(), r.top(), r.width(), color_left_top);
        self.draw_y_line(r.left(), r.top(), r.height(), color_left_top);
        self.draw_x_line(r.left(), r.bottom() - 1, r.width(), color_right_bottom);
        self.draw_y_line(r.right() - 1, r.top(), r.height(), color_right_bottom);
    }

    fn fill_rectangle(&mut self, x : isize, y : isize, w : usize, h : usize, color : Color) {
        let rect = Rect { left_top : Point { x, y }, size : Size { w, h } };
        self.fill_rect(rect, color);
    }

    fn draw_char(&mut self, x : isize, y : isize, c : u8, color : Color, background_color : Color) {
        let c : usize = if c > 127 {0} else {c as usize};

        let mut points_index = 0;
        for iy in y..y + ascii_font::FONT_HEIGHT as isize {
            let points = ascii_font::ASCII_FONT[c][points_index];
            if iy >= 0 && (iy as usize) < self.height() {
                let mut mask : u8 = 0x80;
                for ix in x..x + ascii_font::FONT_WIDTH as isize {
                    if ix >= 0 && (ix as usize) < self.width() {
                        let cc = if(points & mask) !=0 {color} else {background_color};
                        if cc != colors::TRANSPARENT {
                            self.set_pixel(ix as usize, iy as usize, cc);
                        }
                    }
                    mask >>= 1;
                }
            }
            points_index += 1;
        }
    }
    
    fn draw_text(&mut self, x : isize, y : isize, text : &str, color : Color, background_color : Color) {
        let mut x = x;
        for c in text.chars() {
            self.draw_char(x, y, c as u8, color, background_color);
            x += ascii_font::FONT_WIDTH as isize;
        }
    }

    fn draw_line_by_point(&mut self, start : Point<isize>, end : Point<isize>, color : Color){
        for point in Bresenham::new(start, end) {
            self.set_pixel(point.x as usize, point.y as usize, color);
        }
    } 

    #[inline]
    fn draw_line(&mut self, x1 : isize, y1 : isize, x2 : isize, y2 : isize, color : Color){
        self.draw_line_by_point(Point{x:x1,y:y1},Point{x:x2,y:y2},color);
    }

    // #[doc(hidden)]
    // pub fn _print(args: fmt::Arguments) {
    //     use core::fmt::Write;
    //     // WRITER.lock().write_fmt(args).unwrap();
    //     use x86_64::instructions::interrupts;   // new

    //     interrupts::without_interrupts(|| {     // new
    //         WRITER.lock().write_fmt(args).unwrap();
    //     });
    // }

    // #[macro_export]
    // macro_rules! print {
    //     ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
    // }

    // #[macro_export]
    // macro_rules! println {
    //     () => ($crate::print!("\n"));
    //     ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
    // }
}

pub struct ScreenCanvas<'a>
{
    pub dirver : &'a mut dyn crate::device::graphics::GraphicsDriver,
    pub rect : Rect,
}

impl ScreenCanvas<'_> {
    pub fn from_driver(driver : &mut dyn crate::device::graphics::GraphicsDriver) -> ScreenCanvas {
        let rect = driver.get_full_screen_rect();
        ScreenCanvas { dirver: driver, rect: rect }
    }
}

impl Canvas for ScreenCanvas<'_> {
    fn size(&self) -> Size<usize> { self.dirver.get_full_screen_rect().size()}
    
    fn get_pixel(&self, x : usize, y : usize) -> Option<Color> {
        if x<self.rect.width() && y < self.rect.height() {
            self.dirver.get_pixel(self.rect.left() as usize + x, self.rect.top() as usize + y)
        } else {
            None
        }
    }

    fn set_pixel(&mut self, x : usize, y : usize, color : Color) -> bool {
        if x < self.rect.width() && y < self.rect.height() {
            self.dirver.set_pixel(self.rect.left() as usize + x, self.rect.top() as usize + y, color)
        } else {
            false
        }
    }

    fn fill_rect(&mut self, rect : Rect, color : Color) {
        self.dirver.fill_rectangle(rect, color);
    }

    fn fill_rectangle(&mut self, x : isize, y : isize, w : usize, h : usize, color : Color) {
        self.dirver.fill_rectangle(Rect{left_top:Point{x,y},size: Size{w,h} }, color);
    }

    fn draw_line_by_point(&mut self, start : Point<isize>, end : Point<isize>, color : Color){
        self.dirver.draw_line(start, end, color);
    }
}

// struct MemoryCanvas {
//     pixels: Vec<Color>,
//     size: (u32, u32) 
// }

// impl Canvas for MemoryCanvas {
//     fn get_size(&self) -> (u32, u32) {
//         self.size
//     }
    
//     fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
//         self.pixels[(y * self.size.0 + x) as usize] = color;
//     }
    
//     fn get_pixel(&self, x: u32, y: u32) -> u32 {
//         self.pixels[(y * self.size.0 + x) as usize]
//     }
// }
