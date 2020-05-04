use ast::de_bruijn::{Expr, ExprRef};
use std::slice::Iter;
use typed_arena::Arena;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'expr, 'value> {
    Closure {
        env: Vec<&'value Value<'expr, 'value>>,
        body: ExprRef<'expr>,
    },
}

pub struct Heap<'expr, 'value> {
    arena: Arena<Value<'expr, 'value>>,
}

impl<'expr, 'value> Heap<'expr, 'value> {
    pub fn new() -> Self {
        Heap {
            arena: Arena::new(),
        }
    }

    pub fn alloc<'heap>(&'heap self, val: Value<'expr, 'value>) -> &'value Value<'expr, 'value>
    where
        'heap: 'value,
    {
        self.arena.alloc(val)
    }
}

pub struct Stack<'expr, 'value> {
    vec: Vec<&'value Value<'expr, 'value>>,
}

impl<'expr, 'value> Stack<'expr, 'value> {
    pub fn new() -> Self {
        Stack { vec: Vec::new() }
    }

    pub fn push(&mut self, val: &'value Value<'expr, 'value>) {
        self.vec.push(val)
    }

    pub fn pop(&mut self) -> &'value Value<'expr, 'value> {
        self.vec.pop().unwrap()
    }

    pub fn peek(&self) -> &'value Value<'expr, 'value> {
        self.vec.last().unwrap()
    }

    pub fn iter(&self) -> Iter<&'value Value<'expr, 'value>> {
        self.vec.iter()
    }
}

pub fn eval<'expr, 'heap, 'value>(
    heap: &'heap Heap<'expr, 'value>,
    stack: &mut Stack<'expr, 'value>,
    ctx: &Vec<&'value Value<'expr, 'value>>,
    expr: ExprRef<'expr>,
) -> &'value Value<'expr, 'value>
where
    'heap: 'value,
{
    match expr {
        Expr::Var(0) => stack.peek(),
        Expr::Var(n) => ctx[ctx.len() - n],
        Expr::App(l, r) => {
            let l_value = eval(heap, stack, ctx, l);
            match l_value {
                Value::Closure { env, body } => {
                    let r_value = eval(heap, stack, ctx, r);
                    stack.push(r_value);
                    let res = eval(heap, stack, env, body);
                    stack.pop();
                    res
                }
                _ => panic!("eval failed: expected Closure, got {:?}", l_value),
            }
        }
        Expr::Lam(body) => {
            let mut env = ctx.clone();
            env.extend(stack.iter());
            heap.alloc(Value::Closure { env, body })
        }
    }
}

#[test]
fn test_eval1() {
    let input = &Expr::Lam(&Expr::Var(0));
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::new();
    let mut stack = Stack::new();
    assert_eq!(eval(&mut heap, &mut stack, &Vec::new(), input), output)
}

#[test]
fn test_eval2() {
    let id = &Expr::Lam(&Expr::Var(0));
    let input = &Expr::App(id, id);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::new();
    let mut stack = Stack::new();
    assert_eq!(eval(&mut heap, &mut stack, &Vec::new(), input), output)
}

#[test]
fn test_eval3() {
    let id = &Expr::Lam(&Expr::Var(0));
    let id_value = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let konst = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let input = &Expr::App(konst, id);
    let output = &Value::Closure {
        env: vec![id_value],
        body: &Expr::Var(1),
    };
    let mut heap = Heap::new();
    let mut stack = Stack::new();
    assert_eq!(eval(&mut heap, &mut stack, &Vec::new(), input), output)
}

#[test]
fn test_eval4() {
    let id = &Expr::Lam(&Expr::Var(0));
    let konst = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let konst_id = &Expr::App(konst, id);
    let input = &Expr::App(konst_id, konst);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::new();
    let mut stack = Stack::new();
    assert_eq!(eval(&mut heap, &mut stack, &Vec::new(), input), output)
}
