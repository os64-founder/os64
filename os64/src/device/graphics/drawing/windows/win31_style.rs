use core::cell::RefCell;
use alloc::{rc::{Weak, Rc}, vec, boxed::Box};
use crate::device::graphics::{drawing::{canvas::{Canvas, ScreenCanvas}, colors, ascii_font::FONT_WIDTH}, Point, Rect, Size};
use super::widget_base::{Draw, WidgetBase, add_child};

const ICON_BUTTON_SIZE : usize = 18;
const ICON_CURSOR_SIZE : usize = 20;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ResizeButtonKind {
    None = 0,
    Minimize = 1,
    Maximize = 2,
    Restore = 4,
}

const ALL_RESIZE_BUTTONS : [ResizeButtonKind; 3] = [
    ResizeButtonKind::Restore,
    ResizeButtonKind::Maximize,
    ResizeButtonKind::Minimize,
];

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BorderKind {
    Fixed = 1,
    Resizable = 2,
    Modal = 3,
}

//鼠标指针
pub struct ArrowCursor {}
impl Draw for ArrowCursor {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        let Point{x,y} = rect.left_top;
        let p0 = Point{x, y};
        let p1 = Point{x, y: y + 15};
        let p2 = Point{x: p1.x + 4, y: p1.y - 4};
        let p5 = Point{x: x + 6, y: y + 10};
        let p3 = Point{x: p2.x + 4, y: p2.y + 8};
        let p4 = Point{x: p5.x + 4, y: p5.y + 8};
        let p6 = Point{x: x + 11, y: y + 10};
        let p7 = Point{x: x + 1, y};
        let points = [p0,p1,p2,p3,p4,p5,p6,p7];
        
        for i in 0..points.len()-1 {
            canvas.draw_line_by_point(points[i], points[i+1], colors::BLACK);
        }
    }
}

//创建一个鼠标指针
pub fn create_cursor_widget(position : Point<isize>) -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect : Rect { left_top : position, size : Size { w : ICON_CURSOR_SIZE, h : ICON_CURSOR_SIZE } },
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(ArrowCursor{}),
    })
}

// 最大化/最小化/还原 按钮
pub struct QuickResizeButton{
    pub kind : ResizeButtonKind,
}

///填充一个三角形，用于最小化、最大化、还原按钮
pub fn draw_a_black_triangle<T : Canvas>(canvas: &mut T, start: Point<isize>, down : bool) {
    let color = colors::BLACK;
    match down {
        true => {
            for i in 0..4 {
                canvas.draw_x_line(start.x + i,start.y + i,(7 - i * 2) as usize, color);
            }
        },
        false => {
            for i in 0..4 {
                canvas.draw_x_line(start.x + 3 - i,start.y + i,(1 + i * 2) as usize, color);
            }
        }
    }
}

impl Draw for QuickResizeButton {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        let left_top = rect.left_top;
        let size = Size{w:ICON_BUTTON_SIZE,h:ICON_BUTTON_SIZE};
        canvas.draw_rect_3d( Rect{left_top,size} , colors::WHITE, colors::GREY);
        canvas.fill_rectangle(left_top.x + 1, left_top.y + 1,  ICON_BUTTON_SIZE - 2 , ICON_BUTTON_SIZE - 2,colors::SILVER);
        match self.kind {
            ResizeButtonKind::Minimize => draw_a_black_triangle(canvas, Point{ x : left_top.x + 5, y : left_top.y + 7}, true),
            ResizeButtonKind::Maximize => draw_a_black_triangle(canvas, Point{ x : left_top.x + 5, y : left_top.y + 7}, false),
            ResizeButtonKind::Restore => {
                draw_a_black_triangle(canvas, Point{ x : left_top.x + 5, y : left_top.y + 4}, false);
                draw_a_black_triangle(canvas, Point{ x : left_top.x + 5, y : left_top.y + 10}, true);
            }
            _ => {}
        }
    }
}

//创建一个 最小化/最大化/还原 按钮
pub fn create_resize_button(kind : ResizeButtonKind, left_top : Point<isize>) -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect : Rect { left_top, size : Size { w: ICON_BUTTON_SIZE, h: ICON_BUTTON_SIZE } },
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(QuickResizeButton{kind}),
    })
}

//菜单 按钮
pub struct MenuButton();

impl Draw for MenuButton {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        let left_top = rect.left_top;
        canvas.fill_rectangle(left_top.x, left_top.y, ICON_BUTTON_SIZE, ICON_BUTTON_SIZE, colors::SILVER);
        let w = 13;
        let dx = 2;
        let dy = 7;
        canvas.draw_x_line(left_top.x + dx + 1, left_top.y + dy + 1, w - 2, colors::WHITE);
        canvas.draw_rectangle(left_top.x + dx, left_top.y + dy, w, 3, colors::BLACK);
        canvas.draw_x_line(left_top.x + dx + 1, left_top.y + dy + 3, w - 1, colors::GREY);
        canvas.draw_y_line(left_top.x + dx + w as isize, left_top.y + dy + 1, 3, colors::GREY);
    }
}

//创建 菜单 按钮
pub fn create_menu_button() -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect : Rect { left_top: Point{x:0,y:0},size: Size { w: ICON_BUTTON_SIZE, h: ICON_BUTTON_SIZE }},
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(MenuButton{}),
    })
}

//标题 文字
pub struct TitleLabel<'a> {
    pub title : &'a str,
    pub width : usize,
}

impl Draw for TitleLabel<'_> {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        let startx = ( rect.width() as isize - ( self.title.as_bytes().len() * FONT_WIDTH ) as isize ) / 2;
        canvas.draw_text(rect.left() + 1 + startx, rect.top() + 1, self.title, colors::WHITE, colors::TRANSPARENT);
    }
}

//创建 标题 文字
pub fn create_title_label(title : &'static str, width : usize) -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect : Rect { left_top: Point{x:0,y:0},size: Size { w: width, h: ICON_BUTTON_SIZE }},
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(TitleLabel{title,width}),
    })
}

//标题栏
pub struct TitleBar {
    pub resize_buttons : u8,
    pub width : usize,
}

impl Draw for TitleBar {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        let left_top = rect.left_top;
        let mut right_start = Point{ x: left_top.x + self.width as isize, y: left_top.y };
        for button in ALL_RESIZE_BUTTONS {
            if self.resize_buttons & (button as u8) != 0 {
                right_start.x -= ICON_BUTTON_SIZE as isize;
            }
        }
        let t = Point{ x: left_top.x + ICON_BUTTON_SIZE as isize, y: left_top.y };
        canvas.fill_rectangle(t.x, t.y, (right_start.x - t.x) as usize, ICON_BUTTON_SIZE, colors::BLUE);
    }
}

//创建 标题栏
pub fn create_title_bar(title : &'static str, rect : Rect, resize_buttons : u8) -> Rc<WidgetBase> {
    let title_bar = Rc::new(WidgetBase {
        rect,
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(TitleBar{resize_buttons, width: rect.width()}),
    });

    //左侧 菜单 按钮
    let menu_button = create_menu_button();
    add_child(&title_bar, menu_button);

    //右侧 最大化/最小化/还原 按钮
    let mut right_start = Point{ x: rect.width() as isize, y: 0 };
    for button in ALL_RESIZE_BUTTONS {
        if resize_buttons & (button as u8) != 0 {
            right_start.x -= ICON_BUTTON_SIZE as isize;
            let resize_button = create_resize_button(button, right_start);
            add_child(&title_bar, resize_button);
        }
    }

    let title_label = create_title_label(title, right_start.x as usize - ICON_BUTTON_SIZE);
    add_child(&title_bar, title_label);

    title_bar
}

//窗口内容
pub struct WindowContent ();

impl Draw for WindowContent {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        canvas.fill_rect(rect, colors::WHITE);
    }
}

//创建 窗口内容
pub fn create_window_content(rect : Rect) -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect,
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(WindowContent{}),
    })
}

//窗口
pub struct Window{
    border : BorderKind,
}

impl Draw for Window {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect) {
        match self.border {
            BorderKind::Fixed => {
                canvas.draw_rect(rect, colors::BLACK);
                canvas.draw_x_line(rect.left()+ 1, rect.top() + 1 + ICON_BUTTON_SIZE as isize, rect.width() - 2, colors::BLACK);
            },
            BorderKind::Modal => {
                for i in 0..4 {
                    canvas.draw_rectangle(rect.left() + i, rect.top() + i , rect.width() - (i as usize) * 2, rect.height() - (i as usize) * 2, colors::BLUE);
                }
                canvas.draw_rectangle(rect.left() + 4, rect.top() + 4 , rect.width() - 8, rect.height() - 8, colors::WHITE);
            }, 
            BorderKind::Resizable => {
                let boarder_colors = [colors::BLACK,colors::SILVER,colors::SILVER,colors::BLACK];
                for (i,color) in boarder_colors.iter().enumerate() {
                    canvas.draw_rectangle(rect.left() + i as isize, rect.top() + i as isize, rect.width() - i * 2, rect.height() - i * 2, *color);
                }
                canvas.draw_x_line(rect.left() + 1, rect.bottom() - 4 - ICON_BUTTON_SIZE as isize, 2, colors::BLACK);
                canvas.draw_x_line(rect.right() - 3, rect.bottom() - 4 - ICON_BUTTON_SIZE as isize, 2, colors::BLACK);
                canvas.draw_y_line(rect.left() + 4 + ICON_BUTTON_SIZE as isize, rect.top() + 1, 2, colors::BLACK);
                canvas.draw_y_line(rect.right() - 5 - ICON_BUTTON_SIZE as isize, rect.top() + 1, 2, colors::BLACK);
                canvas.draw_y_line(rect.left() + 4 + ICON_BUTTON_SIZE as isize, rect.bottom() - 3, 2, colors::BLACK);
                canvas.draw_y_line(rect.right() - 5 - ICON_BUTTON_SIZE as isize, rect.bottom() - 3, 2, colors::BLACK);
                canvas.draw_x_line(rect.left()+ 1, rect.top() + 4 + ICON_BUTTON_SIZE as isize, rect.width() - 2, colors::BLACK);
            }
        }
    }
}

//创建 窗口
pub fn create_window(title : &'static str, rect : Rect, border : BorderKind) -> Rc<WidgetBase> {
    let window =     Rc::new( WidgetBase {
        rect,
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(Window{border}),
    } );

    let resize_buttons : u8;
    let border_size : isize;
    let mut rect = rect;

    match border {
        BorderKind::Fixed => {
            resize_buttons = ResizeButtonKind::Minimize as u8;
            border_size = 1;
            rect.left_top.x = 1;
            rect.left_top.y = 2 + ICON_BUTTON_SIZE as isize;
            rect.size.w -= 2;
            rect.size.h -= 3 + ICON_BUTTON_SIZE;
        },
        BorderKind::Modal => {
            resize_buttons = ResizeButtonKind::None as u8;
            border_size = 5;
            rect.left_top.x = 5;
            rect.left_top.y = 5 + ICON_BUTTON_SIZE as isize;
            rect.size.w -= 10;
            rect.size.h -= 10 + ICON_BUTTON_SIZE;
        },
        BorderKind::Resizable => {
            border_size = 4;
            resize_buttons = ResizeButtonKind::Maximize as u8 | ResizeButtonKind::Minimize as u8;
            rect.left_top.x = 4;
            rect.left_top.y = 5 + ICON_BUTTON_SIZE as isize;
            rect.size.w -= 8;
            rect.size.h -= 10 + ICON_BUTTON_SIZE;
        },
    }

    let rect_title = Rect{ left_top : Point{ x : border_size, y : border_size }, size : Size{ w : rect.width(), h : ICON_BUTTON_SIZE } };
    let title_bar = create_title_bar(title, rect_title, resize_buttons);
    let content = create_window_content(rect);

    add_child(&window, title_bar);
    add_child(&window, content);

    window
}

//桌面
pub struct Desktop ();
impl Draw for Desktop {
    fn draw(&self, _ : &mut ScreenCanvas, _ : Rect) {
        // canvas.fill_rect(rect, colors::SILVER);
    }
}

//创建 桌面
pub fn create_desktop(rect : Rect) -> Rc<WidgetBase> {
    Rc::new(WidgetBase {
        rect,
        visble : true,
        enable : true,
        parent : RefCell::new(Weak::new()),
        children : RefCell::new(vec![]),
        component : Box::new(Desktop{}),
    })
}
