pub const RING_BUFFER_SIZE : u32 = 240;

pub struct RingBuffer<T : Default + Copy> {
    pub count: u32,
    pub head: u32,
    pub tail: u32,
    pub buffer: [T; RING_BUFFER_SIZE as usize],
}

impl<T : Default + Copy> RingBuffer<T> {
    pub fn new() -> Self {
        Self {
            count: 0, 
            head: 0,
            tail: 0, 
            buffer: [T::default(); RING_BUFFER_SIZE as usize],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.count < RING_BUFFER_SIZE {
            self.buffer[self.head  as usize] = item;
            self.head = (self.head + 1) % RING_BUFFER_SIZE;
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let item = self.buffer[self.tail as usize];
            self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
            Some(item)
        }
    }
}