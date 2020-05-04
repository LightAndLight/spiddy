use crate::value::Value;
use num::Integer;
use std::alloc::{GlobalAlloc, Layout, System};

pub struct Stack<'expr, 'value> {
    capacity: usize,
    size: usize,
    buffer: *mut &'value Value<'expr, 'value>,
}

pub struct Iter<'expr, 'value> {
    remaining: usize,
    base: *mut &'value Value<'expr, 'value>,
}

impl<'expr, 'value> Iterator for Iter<'expr, 'value> {
    type Item = &'value Value<'expr, 'value>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining {
            0 => Option::None,
            _ => {
                self.remaining -= 1;
                Option::Some(unsafe { *self.base.offset(self.remaining as isize) })
            }
        }
    }
}

impl<'expr, 'value> Stack<'expr, 'value> {
    /// Create a stack with the given capacity in bytes. Panics if the capacity is exceeded.
    pub fn with_capacity(size_bytes: usize) -> Self {
        let (q, r) = size_bytes.div_rem(&std::mem::size_of::<&Value>());
        let size_items = q + match r == 0 {
            true => 0,
            false => 1,
        };
        Stack {
            capacity: size_items,
            size: 0,
            buffer: unsafe {
                System
                    .alloc(Layout::from_size_align_unchecked(
                        size_items,
                        std::mem::align_of::<&Value>(),
                    ))
                    .cast()
            },
        }
    }

    pub fn push(&mut self, val: &'value Value<'expr, 'value>) {
        if self.size == self.capacity {
            panic!("Stack::push failed: stack overflow")
        }
        unsafe { *self.buffer.offset(self.size as isize) = val };
        self.size += 1;
    }

    pub fn pop(&mut self) -> &'value Value<'expr, 'value> {
        self.size -= 1;
        unsafe { *self.buffer.offset(self.size as isize) }
    }

    pub fn peek(&self) -> &'value Value<'expr, 'value> {
        unsafe { *self.buffer.offset(self.size as isize - 1) }
    }

    pub fn iter(&self) -> Iter<'expr, 'value> {
        Iter {
            remaining: self.size,
            base: self.buffer,
        }
    }
}
