use crate::value::Value;

use num::Integer;
use typed_arena::Arena;

pub struct Heap<'expr, 'value> {
    arena: Arena<Value<'expr, 'value>>,
}

impl<'expr, 'value> Heap<'expr, 'value> {
    /// Create a heap with the given initial capacity in bytes. Grows if the capacity is exceeded.
    pub fn with_capacity(size_bytes: usize) -> Self {
        let (q, r) = size_bytes.div_rem(&std::mem::size_of::<Value>());
        let size_items = q + match r == 0 {
            true => 0,
            false => 1,
        };
        Heap {
            arena: Arena::with_capacity(size_items),
        }
    }

    pub fn alloc<'heap>(&'heap self, val: Value<'expr, 'value>) -> &'value Value<'expr, 'value>
    where
        'heap: 'value,
    {
        self.arena.alloc(val)
    }
}
