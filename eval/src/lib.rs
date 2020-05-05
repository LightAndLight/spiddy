pub mod heap;
pub mod stack;
pub mod value;

use crate::heap::Heap;
use crate::value::Value;
use ast::de_bruijn::{Expr, ExprRef};

pub fn eval<'expr, 'heap, 'value>(
    heap: &'heap Heap<'expr, 'value>,
    env: &Vec<&'value Value<'expr, 'value>>,
    expr: ExprRef<'expr>,
) -> &'value Value<'expr, 'value>
where
    'heap: 'value,
{
    let res = match expr {
        Expr::Var(n) => env[env.len() - n - 1],
        Expr::App(l, r) => {
            let l_value = eval(heap, env, l);
            match l_value {
                Value::Closure { env: next, body } => {
                    let r_value = eval(heap, env, r);

                    let mut env = next.clone();
                    env.push(r_value);
                    let res = eval(heap, &env, body);
                    res
                }
                _ => panic!("eval failed: expected Closure, got {:?}", l_value),
            }
        }
        Expr::Lam(body) => heap.alloc(Value::Closure {
            env: env.clone(),
            body: body,
        }),
        Expr::U64(n) => heap.alloc(Value::U64(*n)),
        Expr::AddU64(l, r) => {
            let lvalue = eval(heap, env, l);
            match lvalue {
                Value::U64(l_n) => {
                    let rvalue = eval(heap, env, r);

                    match rvalue {
                        Value::U64(r_n) => heap.alloc(Value::U64(l_n + r_n)),
                        r_value => panic!("eval failed: expected U64, got {:?}", r_value),
                    }
                }
                l_value => panic!("eval failed: expected U64, got {:?}", l_value),
            }
        }
    };
    res
}

type Env<'expr, 'value> = Vec<&'value Value<'expr, 'value>>;
type ValueRef<'expr, 'value> = &'value Value<'expr, 'value>;

#[derive(Debug)]
enum Hole {
    Hole,
}

/// The meaning of `Cont` is a function from `ValueRef -> ValueRef`
#[derive(Debug)]
enum Cont<'expr, 'value> {
    AppL(Env<'expr, 'value>, Hole, ExprRef<'expr>),
    AppR(Env<'expr, 'value>, ExprRef<'expr>, Hole),
    AddU64L(Env<'expr, 'value>, Hole, ExprRef<'expr>),
    AddU64R(u64, Hole),
}

#[derive(Debug)]
enum Code<'expr, 'value> {
    Input(ExprRef<'expr>),
    Output(ValueRef<'expr, 'value>),
}

pub fn eval_loop<'expr, 'heap, 'value>(
    heap: &'heap Heap<'expr, 'value>,
    expr: ExprRef<'expr>,
) -> ValueRef<'expr, 'value>
where
    'heap: 'value,
{
    use crate::Code::*;
    use crate::Cont::*;
    use crate::Hole::*;

    let mut env: Env<'expr, 'value> = Vec::new();
    let mut code: Code<'expr, 'value> = Input(expr);
    let mut cont: Vec<Cont<'expr, 'value>> = Vec::new();
    loop {
        // println!("C: {:?}", code);
        // println!("E: {:?}", env);
        // println!("K: {:?}", cont);
        // println!("---------------------------------");
        match code {
            Input(expr) => match expr {
                Expr::U64(n) => {
                    code = Output(heap.alloc(Value::U64(*n)));
                }
                Expr::Var(n) => {
                    code = Output(env[env.len() - n - 1]);
                }
                Expr::App(l, r) => {
                    code = Input(l);
                    cont.push(AppL(env.clone(), Hole, r));
                }
                Expr::Lam(body) => {
                    code = Output(heap.alloc(Value::Closure {
                        env: env.clone(),
                        body: body,
                    }));
                }
                Expr::AddU64(l, r) => {
                    code = Input(l);
                    cont.push(AddU64L(env.clone(), Hole, r));
                }
            },
            Output(value) => match cont.pop() {
                Option::None => match code {
                    Input(_) => panic!("eval_loop failed: no output to return"),
                    Output(value) => {
                        return value;
                    }
                },
                Option::Some(c) => match c {
                    AppL(r_env, Hole, r) => match value {
                        Value::Closure { env: l_env, body } => {
                            code = Input(r);
                            env = r_env;
                            cont.push(AppR(l_env.clone(), body, Hole));
                        }
                        _ => panic!("eval_loop failed: Expected closure, got {:?}", value),
                    },
                    AppR(next_env, body, Hole) => {
                        let mut next_env = next_env;
                        next_env.push(value);

                        env = next_env;
                        code = Input(body);
                    }
                    AddU64L(r_env, Hole, r) => match value {
                        Value::U64(l) => {
                            code = Input(r);
                            env = r_env;
                            cont.push(AddU64R(*l, Hole));
                        }
                        _ => panic!("eval_loop failed: Expected u64, got {:?}", value),
                    },
                    AddU64R(l, Hole) => match value {
                        Value::U64(r) => {
                            code = Output(heap.alloc(Value::U64(l + r)));
                        }
                        _ => panic!("eval_loop failed: Expected u64, got {:?}", value),
                    },
                },
            },
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
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
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
    assert_eq!(eval(&mut heap, &Vec::new(), input), output)
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
        body: &Expr::Var(0),
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

#[test]
fn test_eval_loop1() {
    let input = &Expr::Lam(&Expr::Var(0));
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval_loop(&mut heap, input), output)
}

#[test]
fn test_eval_loop2() {
    let id = &Expr::Lam(&Expr::Var(0));
    let input = &Expr::App(id, id);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval_loop(&mut heap, input), output)
}

#[test]
fn test_eval_loop3() {
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
    assert_eq!(eval_loop(&mut heap, input), output)
}

#[test]
fn test_eval_loop4() {
    let id = &Expr::Lam(&Expr::Var(0));
    let konst = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let konst_id = &Expr::App(konst, id);
    let input = &Expr::App(konst_id, konst);
    let output = &Value::Closure {
        env: Vec::new(),
        body: &Expr::Var(0),
    };
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval_loop(&mut heap, input), output)
}

#[test]
fn test_eval_loop5() {
    let plus = &Expr::Lam(&Expr::Lam(&Expr::AddU64(&Expr::Var(0), &Expr::Var(1))));
    let plus_9 = &Expr::App(plus, &Expr::U64(9));
    let input = &Expr::App(plus_9, &Expr::U64(7));
    let output = &Value::U64(16);
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval_loop(&mut heap, input), output)
}

#[test]
fn test_eval_loop6() {
    let plus = &Expr::Lam(&Expr::Lam(&Expr::AddU64(&Expr::Var(0), &Expr::Var(1))));
    let apply_9_7 = &Expr::Lam(&Expr::App(
        &Expr::App(&Expr::Var(0), &Expr::U64(9)),
        &Expr::U64(7),
    ));
    let input = &Expr::App(apply_9_7, plus);
    let output = &Value::U64(16);
    let mut heap = Heap::with_capacity(1024);
    assert_eq!(eval_loop(&mut heap, input), output)
}
