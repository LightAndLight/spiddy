use crate::value::Value;
use num::Integer;
use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Debug;
use std::ops::Index;

pub struct Stack<'expr, 'value> {
    capacity: usize,
    size: usize,
    buffer: *mut &'value Value<'expr, 'value>,
}

impl<'expr, 'value> Index<usize> for Stack<'expr, 'value> {
    type Output = &'value Value<'expr, 'value>;
    fn index<'stack>(&'stack self, ix: usize) -> &'stack Self::Output {
        unsafe { &*self.buffer.offset(self.size as isize - ix as isize - 1) }
    }
}

impl<'expr, 'value> Debug for Stack<'expr, 'value> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.iter_from_bottom().collect::<Vec<_>>().fmt(formatter)
    }
}

pub struct IterFromTop<'expr, 'value> {
    remaining: usize,
    base: *mut &'value Value<'expr, 'value>,
}

impl<'expr, 'value> Iterator for IterFromTop<'expr, 'value> {
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

pub struct IterFromBottom<'expr, 'value> {
    current: usize,
    size: usize,
    base: *mut &'value Value<'expr, 'value>,
}

impl<'expr, 'value> Iterator for IterFromBottom<'expr, 'value> {
    type Item = &'value Value<'expr, 'value>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.size {
            Option::None
        } else {
            self.current += 1;
            Option::Some(unsafe { *self.base.offset(self.current as isize) })
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

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iter_from_top(&self) -> IterFromTop<'expr, 'value> {
        IterFromTop {
            remaining: self.size,
            base: self.buffer,
        }
    }

    pub fn iter_from_bottom(&self) -> IterFromBottom<'expr, 'value> {
        IterFromBottom {
            size: self.size,
            current: 0,
            base: self.buffer,
        }
    }
}

#[test]
fn test_stack1() {
    let mut stack = Stack::with_capacity(1024);
    stack.push(&Value::U64(999));
    assert_eq!(stack[0], &Value::U64(999));
    stack.push(&Value::U64(10));
    assert_eq!(stack[0], &Value::U64(10));
    assert_eq!(stack[1], &Value::U64(999));
    stack.push(&Value::U64(42));
    assert_eq!(stack[0], &Value::U64(42));
    assert_eq!(stack[1], &Value::U64(10));
    assert_eq!(stack[2], &Value::U64(999));
}
