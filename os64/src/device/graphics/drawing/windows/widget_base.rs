use core::cell::RefCell;
use alloc::{vec::Vec, rc::{Weak, Rc}, boxed::Box};
use futures_util::task::WakerRef;
use crate::device::graphics::{Point, drawing::{canvas::{Canvas, ScreenCanvas}, colors}, Rect, Size};

pub const IS_DEBUG_MODE : bool = false;

pub trait Draw {
    fn draw(&self, canvas : &mut ScreenCanvas, rect : Rect);
}

pub trait Widget {
    fn get_position(&self) -> Point<isize>;
    fn set_position(&mut self, value : Point<isize>);

    fn get_size(&self) -> Size<usize>;
    fn set_size(&mut self, value : Size<usize>);

    fn get_rect(&self) -> Rect { Rect { left_top : self.get_position(), size : self.get_size() } }
    fn set_rect(&mut self, value : Rect);

    fn get_visible(&self) -> bool;
    fn set_visible(&mut self, value : bool);

    fn get_enable(&self) -> bool;
    fn set_enable(&mut self, value : bool);

    // fn get_parent(&self) -> RefCell<Weak<dyn Widget>>;
    // fn set_parent(&mut self, value : RefCell<Weak<dyn Widget>>);

    // fn get_children(&mut self) -> RefCell<Vec<Box<dyn Widget>>>;

    fn draw(&self, canvas : &mut ScreenCanvas, position : Point<isize>);
	fn repaint(&self, canvas :&mut ScreenCanvas, position : Point<isize>, r : Rect);

    /// 控件内坐标的点到屏幕对应坐标的转换
	fn local_point_to_screen(&self, p: Point<isize>) -> Point<isize>;
    
    /// 屏幕坐标的点到控件内对应坐标的转换
	fn screen_point_to_local(&self,  p : Point<isize>) -> Point<isize>;

    // fn remove_child(&mut self, child: Box<dyn Widget>);
    // fn add_child(&mut self, child: Box<dyn Widget>);
    // fn insert_child(&mut self, child: Box<dyn Widget>);
}

pub struct WidgetBase {
    pub rect : Rect,
    pub visble : bool,
    pub enable : bool,    
    pub parent : RefCell<Weak<WidgetBase>>,
    pub children : RefCell<Vec<Rc<WidgetBase>>>,
    pub component : Box<dyn Draw>,
}

impl Widget for WidgetBase {
    fn get_position(&self) -> Point<isize> {
        self.rect.left_top
    }

    fn set_position(&mut self, value : Point<isize>) {
        self.rect.left_top = value;
    }

    fn get_size(&self) -> Size<usize> {
        self.rect.size
    }

    fn set_size(&mut self, value : Size<usize>) {
        self.rect.size = value;
    }

    fn get_rect(&self) -> Rect { 
        self.rect
    }

    fn set_rect(&mut self, value : Rect) {
        self.rect = value;
    }

    fn get_visible(&self) -> bool {
        self.visble
    }

    fn set_visible(&mut self, value : bool) {
        self.visble = value;
    }

    fn get_enable(&self) -> bool {
        self.enable
    }

    fn set_enable(&mut self, value : bool) {
        self.enable = value;
    }

    fn draw(&self, canvas : &mut ScreenCanvas, position : Point<isize>) {
        if IS_DEBUG_MODE {
            // paint.setColor(Color.RED);
            // paint.setStyle(Paint.Style.STROKE);
            canvas.draw_rect(Rect{left_top: position, size: self.get_size()},colors::RED);//.GetAndroidRect());
        }
        let mut rect = self.rect;
        rect.left_top.x = position.x;
        rect.left_top.y = position.y;
        self.component.draw(canvas, rect);
    }

    fn local_point_to_screen(&self, p: Point<isize>) -> Point<isize> {
        match self.parent.borrow_mut().upgrade() {
            Some(parent) =>{
                let mut p = p;
                p.x += parent.get_rect().left();
                p.y += parent.get_rect().top();
                parent.local_point_to_screen(p)
            },
            None => {
                // panic!("parent is destoried");
                p
            }
        }
    }

    fn screen_point_to_local(&self,  p : Point<isize>) -> Point<isize> {
        match self.parent.borrow_mut().upgrade() {
            Some(parent) => {
                let mut p = p;
                p.x -= parent.get_rect().left();
                p.y -= parent.get_rect().top();
                parent.screen_point_to_local(p)
            },
            None => {
                // panic!("parent is destoried");
                p
            }
        }
    }

    // fn get_parent(&self) -> RefCell<Weak<dyn Widget>> {
    //     self.parent.clone()
    // }

    // fn set_parent(&mut self, value : RefCell<Weak<WidgetBase>>) {
    //     self.parent = value;
    // }

    // fn get_children(&mut self) -> Vec<Box<dyn Widget>> {
    //     self.children
    // }

    /// 重画处理
    /// @param canvas 画布
    /// @param baseX 基于屏幕左上角该对象的基准位置横坐标
    /// @param baseY 基于屏幕左上角该对象的基准位置纵坐标
    /// @param r 本对象rect与本次无效矩形的交集(r.lefttop的坐标基于本对象)
	fn repaint(&self, canvas :&mut ScreenCanvas, position : Point<isize>, r : Rect) {
        if self.get_visible() {
           // 设置裁剪区
            // if(!DebugMode)
            // {
            //     canvas.clipRect
            //         ( baseX + r.lefttop.x
            //         , baseY + r.lefttop.y
            //         , baseX + r.size.w + r.lefttop.x
            //         , baseY + r.size.h + r.lefttop.y
            //         , Region.Op.REPLACE
            //         );
            // }
            self.draw(canvas, position);            
            for child in self.children.borrow_mut().iter_mut() {
                //可见、并且和无效矩形范围相交才画
                if child.get_visible() {
                    let ir = child.get_rect();
                    let rr = r.join(ir);
                    if rr != Rect::EMPTY || IS_DEBUG_MODE {
                        rr.left_top().x -= ir.left_top().x;
                        rr.left_top().y -= ir.left_top().y;
                        let child_pos = Point{ x: position.x + ir.left() , y: position.y + ir.top() };
                        child.repaint(canvas, child_pos, r);
                    }
                }
            }
        }
    }
}

impl  WidgetBase {
    /// 移除一个子对象
    fn remove_child(&mut self, child: Rc<WidgetBase>) {
        let idx = self.children.borrow().iter().position(|c| Rc::as_ptr(&child)==Rc::as_ptr(c));
        self.children.borrow_mut().remove(idx.unwrap());
        *child.parent.borrow_mut() = Weak::new();
    }

    // /// 增加一个子对象
    // fn add_child(&self, child: Rc<WidgetBase>) {
    //     *child.parent.borrow_mut() = Weak::from(self);
    //     self.children.borrow_mut().push(child);
    // }

    // /// 插入一个子对象
    // fn insert_child(&self, child: Rc<WidgetBase>) {
    //     *child.parent.borrow_mut() = Rc::downgrade(self);
    //     self.children.borrow_mut().insert(0,child);
    // }
}


/// 增加一个子对象
pub fn add_child(parent:& Rc<WidgetBase>, child: Rc<WidgetBase>) {
    *child.parent.borrow_mut() = Rc::downgrade(&parent);
    parent.children.borrow_mut().push(child);
}

/// 插入一个子对象
pub fn insert_child(parent:& Rc<WidgetBase>, child: Rc<WidgetBase>) {
    *child.parent.borrow_mut() = Rc::downgrade(&parent);
    parent.children.borrow_mut().insert(0,child);
}

// /// 移除一个子对象
// fn remove_child(parent:& Rc<WidgetBase>, child: Rc<WidgetBase>) {
//     let idx = parent.children.borrow().iter().position(|c| Rc::as_ptr(&child)==Rc::as_ptr(c));
//     parent.children.borrow_mut().remove(idx.unwrap());
//     *child.parent.borrow_mut() = Weak::new();
// }    

//     //
//     // 触笔遍历
//     //   因为画控件是从 firstchild 往 lastchild 画
//     //   越靠近 lastchild 的控件会越不容易被遮挡住
//     //   因此触笔事件从 lastchild 往 firstchild 遍历
//     //
// 	// private boolean PenDownForEach(WindowsBase subObject,int x,int y)
//     // {
//     //     PenDownStack.push(subObject);
//     //     //MyLog.d(null, "PenDown=======obj="+subObject+",penDown.X="+x+",penDown.Y="+y);
//     //     if(subObject.penEvent!=null && subObject.penEvent.DoPenDown(subObject,x,y))
//     //     {
//     //         for(int i = subObject.children.size() - 1; i >= 0 ; i--)
//     //         {
//     //             WindowsBase sub = subObject.children.elementAt(i);
//     //             if(sub!=null && sub.visible && sub.enabled && sub.rect.ContainsPoint(x,y))
//     //             {
//     //                 if(!this.PenDownForEach(sub, x - sub.getLeft(), y - sub.getTop()))
//     //                 {
//     //                     return false;
//     //                 }
//     //                 break;
//     //             }
//     //         }
//     //     }
//     //     return true;
//     // }
    
// 	// public boolean PenDown(int x,int y)
//     // {
//     //     PenDownStack.clear();
//     //     return PenDownForEach(this,x,y);
//     // }
	
//     // public boolean PenUp(int x,int y)
//     // {
//     //     //每一个收到PenDown事件的控件都会收到PenUp事件,因此无需使用回调
//     //     int i=0;
//     //     int l = PenDownStack.size();
//     //     //while(!PenDownStack.empty())
//     //     for(;i<l;i++)
//     //     {
//     //         WindowsBase obj = PenDownStack.elementAt(i);
//     //         int newX = x - obj.rect.Left();
//     //         int newY = y - obj.rect.Top();
//     //         //MyLog.d(null, "PenUp=======obj="+obj+",penUp.X="+ newX +",penUp.Y="+ newY);
//     //         if(obj.penEvent!=null && obj.penEvent.DoPenUp(obj,newX,newY))
//     //         {
//     //             //
//     //         }
//     //         if(obj.rect.ContainsPoint(x,y))
//     //         {
//     //             //很多控件的Click事件会进行返回界面操作，将会导致CurrentPenDown中的数据无效
//     //             //因此需要break，避免出现指针错误，但这也意味着只能执行控件树最底层的一次Click事件
//     //             if(obj.penClickEvent!=null && obj.penClickEvent.DoPenClick(obj, newX,newY))
//     //             {
//     //                 x = newX;
//     //                 y = newY;
//     //                 continue;
//     //             }
//     //         }
//     //         x = newX;
//     //         y = newY;
//     //     }
//     //     PenDownStack.clear();
//     //     return true;
//     // }

//     // public boolean PenMove(int x,int y)
//     // {
//     //     //每一个收到PenDown事件的控件都会收到PenMove事件,因此无需使用回调	
//     //     for(int i=0;i<PenDownStack.size();i++)
//     //     {
//     //         WindowsBase obj = PenDownStack.get(i);
//     //         x -=  obj.getLeft();
//     //         y -=  obj.getTop();
//     //         //MyLog.d(null, "PenMove=======obj="+obj+",penMove.X="+x+",penMove.Y="+y);
//     //         if(obj.penEvent!=null)
//     //         {
//     //             obj.penEvent.DoPenMove(obj, x, y);
//     //         }
//     //     }
//     //     return true;
//     // }
// }
