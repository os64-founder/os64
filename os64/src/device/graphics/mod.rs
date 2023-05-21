use core::cmp;

use self::{algorithm::Bresenham};

pub mod algorithm;
pub mod drawing;
pub mod vga;

use core::cmp::{max, min};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Point<T> {
    pub x : T,
    pub y : T,
}

impl Point<isize> {
    pub const ZERO : Point<isize> = Point{ x : 0, y : 0};
    pub const INFINITE : Point<isize> = Point{ x : isize::MIN, y : isize::MIN};

    pub fn inside(&self, rect : &Rect) -> bool {
        rect.contains(self)
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Size<T> {
    pub w : T,
    pub h : T,
}

impl Size<usize> {
    pub const EMPTY : Size<usize> = Size{ w : 0, h : 0};
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Rect {
    pub left_top : Point<isize>,
    pub size : Size<usize>,
}

impl Rect {
    pub const EMPTY : Rect = Rect{left_top : Point::INFINITE, size : Size::EMPTY};

    pub fn size(&self) -> Size<usize> {self.size}
    pub fn left_top(&self) -> Point<isize> {self.left_top}
    pub fn left(&self) -> isize {self.left_top.x}
    pub fn top(&self) -> isize {self.left_top.y}
    pub fn width(&self) -> usize {self.size.w}
    pub fn height(&self) -> usize {self.size.h}
    pub fn right(&self) -> isize {self.left_top.x + self.size.w as isize}
    pub fn bottom(&self) -> isize {self.left_top.y + self.size.h as isize}

    pub fn join(&self, rect : Rect) -> Rect {
        let left = max(self.left() ,rect.left());
        let top = max(self.top() ,rect.top());
        let right = min(self.right() ,rect.right());
        let bottom = min(self.bottom() ,rect.bottom());
        if left<=right && top<=bottom {
            Rect{left_top:Point{x:left,y:top},size:Size{w:(right-left) as usize,h:(bottom-top) as usize}}
        } else {
            Rect::EMPTY
        }
    }

    pub fn contains(&self, point : &Point<isize>) -> bool {
        point.x >= self.left() && point.x < self.right() && point.y >= self.right() && point.y < self.right()
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Color8 {
    pub value : u8
}

// #[derive(Clone,Copy,Debug)]
// pub struct Color555
// {
//     pub value : u16
// }

// #[derive(Clone,Copy,Debug)]
// pub struct Color565
// {
//     pub value : u16
// }

// #[derive(Clone,Copy,Debug)]
// pub struct Color24
// {
//     pub red : u8 , 
//     pub green : u8 , 
//     pub blue: u8 ,
// }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Color
{
    pub value: u32,
}

// #[derive(Clone,Copy,Debug)]
// pub struct Color64
// {
//     pub alpha : u16 ,
//     pub red : u16 ,
//     pub green : u16 ,
//     pub blue: u16 
// }
// impl Copy for Color64 {}

impl Color {
    pub fn from_argb(alpha: u8, red: u8, green: u8, blue: u8) -> Color {
        let value = blue as u32 | (green as u32) << 8 | (red as u32) << 16 | (alpha as u32) << 24;
        Color { value }
    }

    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Color {
        let value = blue as u32 | (green as u32)<< 8 | (red as u32) << 16 ;
        Color{value}
    }

    pub fn from_u32(value : u32) -> Color {
        Color{value}
    }

    pub fn from_color8(color: Color8) -> Color {
        Color::from_rgb(((color.value >> 4) & 0x3) << 6, ((color.value >> 2) & 0x3) << 6 , (color.value & 0x3) << 6)
    } 

    pub fn alpha(&self) -> u8 {(self.value >> 24) as u8}
    pub fn red(&self) -> u8 {(self.value >> 16) as u8}
    pub fn green(&self) -> u8 {(self.value >> 8) as u8}
    pub fn blue(&self) -> u8 {self.value as u8}
}

impl From<Color> for Color8 {
    fn from(color: Color) -> Color8 {
        Color8 {
            value:{     
                let mut index : u8 = 0;
                let b = color.blue();
                let g = color.green();
                let r = color.red();
                index |= (b >> 7 << 3) as u8; 
                index |= ((b >> 6) & 1) as u8; 
                index |= (g >> 7 << 4) as u8; 
                index |= (((g >> 6) & 1) << 1) as u8; 
                index |= (r >> 7 << 5) as u8; 
                index |= (((r >> 6) & 1) << 2) as u8; 
                index
            }
        }
    }
}


#[derive(Clone,Copy,Debug)]
pub struct DisplayMode {
    pub width: usize,
    pub height: usize,
    pub bpp: u8  // 比特深度
}

pub fn force_between<T : Ord>(value : T, min : T, max : T) -> T {
    cmp::max( cmp::min( value, max ), min)
}

pub trait GraphicsDriver {
    fn init(&mut self); 
 
    fn get_full_screen_size(&self) -> Size<usize>;
    fn get_full_screen_rect(&self) -> Rect {
        Rect { left_top : Point::ZERO , size : self.get_full_screen_size() }
    }

    fn get_pixel(&self, x : usize, y : usize) -> Option<Color> ;
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> bool;

    fn fill_rectangle(&mut self, rect : Rect, color : Color) {
        let rect = self.get_full_screen_rect().join(rect);
        if rect != Rect::EMPTY {
            for iy in rect.top()..rect.bottom() {
                for ix in rect.left()..rect.right() {
                    self.set_pixel(ix as usize,iy as usize,color);
                }
            }
        }
    }
    
    /// Color::from_u32(0x3F7FFF)
    fn clear_screen(&mut self, color: Color) {
        self.fill_rectangle(self.get_full_screen_rect(),  color);
    }

    fn draw_line(&mut self, start : Point<isize>, end : Point<isize>, color : Color){
        for point in Bresenham::new(start, end) {
            self.set_pixel(point.x as usize, point.y as usize, color);
        }
    } 
}
