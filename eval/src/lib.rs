pub mod heap;
pub mod stack;
pub mod value;

use crate::heap::Heap;
use crate::value::Value;
use ast::de_bruijn::{Expr, ExprRef};

pub fn eval<'expr, 'heap, 'value>(
    heap: &'heap Heap<'expr, 'value>,
    ctx: &Vec<&'value Value<'expr, 'value>>,
    expr: ExprRef<'expr>,
) -> &'value Value<'expr, 'value>
where
    'heap: 'value,
{
    let res = match expr {
        Expr::Var(n) => ctx[ctx.len() - n - 1],
        Expr::App(l, r) => {
            let l_value = eval(heap, ctx, l);
            match l_value {
                Value::Closure { env, body } => match body {
                    Expr::Lam(body) => {
                        let r_value = eval(heap, ctx, r);
                        let mut ctx = env.clone();
                        ctx.push(r_value);
                        let res = eval(heap, &ctx, body);
                        res
                    }
                    _ => panic!("eval failed: expected Lam, got {:?}", body),
                },
                _ => panic!("eval failed: expected Closure, got {:?}", l_value),
            }
        }
        Expr::Lam(_) => heap.alloc(Value::Closure {
            env: ctx.clone(),
            body: expr,
        }),
        Expr::U64(n) => heap.alloc(Value::U64(*n)),
        Expr::AddU64(l, r) => match eval(heap, ctx, l) {
            Value::U64(l_n) => match eval(heap, ctx, r) {
                Value::U64(r_n) => heap.alloc(Value::U64(l_n + r_n)),
                r_value => panic!("eval failed: expected U64, got {:?}", r_value),
            },
            l_value => panic!("eval failed: expected U64, got {:?}", l_value),
        },
    };
    res
}

#[test]
fn test_eval1() {
    let input = &Expr::Lam(&Expr::Var(0));
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Lam(&Expr::Var(0)),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}

#[test]
fn test_eval2() {
    let id = &Expr::Lam(&Expr::Var(0));
    let input = &Expr::App(id, id);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Lam(&Expr::Var(0)),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}

#[test]
fn test_eval3() {
    let id = &Expr::Lam(&Expr::Var(0));
    let id_value = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Lam(&Expr::Var(0)),
    };
    let konst = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let input = &Expr::App(konst, id);
    let output = &Value::Closure {
        env: vec![id_value],
        body: &Expr::Lam(&Expr::Var(1)),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}

#[test]
fn test_eval4() {
    let id = &Expr::Lam(&Expr::Var(0));
    let konst = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let konst_id = &Expr::App(konst, id);
    let input = &Expr::App(konst_id, konst);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Lam(&Expr::Var(0)),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}

#[test]
fn test_eval5() {
    let plus = &Expr::Lam(&Expr::Lam(&Expr::AddU64(&Expr::Var(0), &Expr::Var(1))));
    let plus_9 = &Expr::App(plus, &Expr::U64(9));
    let input = &Expr::App(plus_9, &Expr::U64(7));
    let output = &Value::U64(16);
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}

#[test]
fn test_eval6() {
    let plus = &Expr::Lam(&Expr::Lam(&Expr::AddU64(&Expr::Var(0), &Expr::Var(1))));
    let apply_9_7 = &Expr::Lam(&Expr::App(
        &Expr::App(&Expr::Var(0), &Expr::U64(9)),
        &Expr::U64(7),
    ));
    let input = &Expr::App(apply_9_7, plus);
    let output = &Value::U64(16);
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
}
