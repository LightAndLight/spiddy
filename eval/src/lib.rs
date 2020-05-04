pub mod heap;
pub mod stack;
pub mod value;

use crate::heap::Heap;
use crate::stack::Stack;
use crate::value::Value;
use ast::de_bruijn::{Expr, ExprRef};

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
    let mut heap = Heap::with_capacity(1024);
    let mut stack = Stack::with_capacity(64);
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
    let mut heap = Heap::with_capacity(1024);
    let mut stack = Stack::with_capacity(64);
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
    let mut heap = Heap::with_capacity(1024);
    let mut stack = Stack::with_capacity(64);
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
    let mut heap = Heap::with_capacity(1024);
    let mut stack = Stack::with_capacity(64);
    assert_eq!(eval(&mut heap, &mut stack, &Vec::new(), input), output)
}
