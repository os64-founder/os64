use core::ops::{Neg, Sub};
use num_traits::Zero;

use crate::device::graphics::Point;

/// A simple octant struct for transforming line points.
pub struct Octant {
    value: u8,
}

impl Octant {
    #[inline]
    /// Get the relevant octant from a start and end point.
    pub fn new<T>(start: Point<T>, end: Point<T>) -> Self
    where
        T: Sub<Output = T> + Neg<Output = T> + PartialOrd + Zero,
    {
        let mut value = 0;
        let mut dx = end.x - start.x;
        let mut dy = end.y - start.y;

        if dy < T::zero() {
            dx = -dx;
            dy = -dy;
            value += 4;
        }

        if dx < T::zero() {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            value += 2;
        }

        if dx < dy {
            value += 1;
        }

        Self { value }
    }

    /// Convert a point to its position in the octant.
    #[inline]
    pub fn to<T>(&self, point: Point<T>) -> Point<T>
    where
        T: Neg<Output = T>,
    {
        match self.value {
            0 => Point{ x:point.x, y:point.y },
            1 => Point{ x:point.y, y:point.x },
            2 => Point{ x:point.y, y:-point.x },
            3 => Point{ x:-point.x, y:point.y },
            4 => Point{ x:-point.x, y:-point.y },
            5 => Point{ x:-point.y, y:-point.x },
            6 => Point{ x:-point.y, y:point.x },
            7 => Point{ x:point.x, y:-point.y },
            _ => unreachable!(),
        }
    }

    /// Convert a point from its position in the octant.
    #[inline]
    pub fn from<T: Neg<Output = T>>(&self, point: Point<T>) -> Point<T> {
        match self.value {
            0 => Point{ x:point.x, y:point.y },
            1 => Point{ x:point.y, y:point.x },
            2 => Point{ x:-point.y, y:point.x },
            3 => Point{ x:-point.x, y:point.y },
            4 => Point{ x:-point.x, y:-point.y },
            5 => Point{ x:-point.y, y:-point.x },
            6 => Point{ x:point.y, y:-point.x },
            7 => Point{ x:point.x, y:-point.y },
            _ => unreachable!(),
        }
    }
}
