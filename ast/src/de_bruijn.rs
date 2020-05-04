use crate::syntax;
use std::collections::HashMap;
use typed_arena::Arena;

fn __from_ast<'src, 'ast, 'builder, 'expr>(
    var_map: &mut HashMap<&'src str, Vec<usize>>,
    builder: &'builder ExprBuilder<'expr>,
    expr: syntax::ExprRef<'src, 'ast>,
) -> ExprRef<'expr>
where
    'builder: 'expr,
{
    match expr {
        syntax::Expr::Parens(inner) => __from_ast(var_map, builder, inner),
        syntax::Expr::Ident(ident) => builder.mk_var(*var_map.get(ident).unwrap().last().unwrap()),
        syntax::Expr::App(l, r) => builder.mk_app(
            __from_ast(var_map, builder, l),
            __from_ast(var_map, builder, r),
        ),
        syntax::Expr::Lam(arg, body) => {
            for value in var_map.values_mut() {
                value[0] += 1;
            }
            match var_map.get_mut(arg) {
                Option::Some(value) => {
                    value.push(0);
                }
                Option::None => {
                    var_map.insert(arg, vec![0]);
                }
            }
            let res = builder.mk_lam(__from_ast(var_map, builder, body));
            match var_map.get_mut(arg) {
                Option::Some(value) => {
                    if value.len() <= 1 {
                        var_map.remove(arg);
                    } else {
                        value.pop();
                    }
                }
                Option::None => {}
            }
            for value in var_map.values_mut() {
                value[0] -= 1;
            }
            res
        }
    }
}

pub fn from_ast<'src, 'ast, 'builder, 'expr>(
    builder: &'builder ExprBuilder<'expr>,
    expr: syntax::ExprRef<'src, 'ast>,
) -> ExprRef<'expr>
where
    'builder: 'expr,
{
    let mut var_map = HashMap::new();
    __from_ast(&mut var_map, builder, expr)
}

pub type ExprRef<'expr> = &'expr Expr<'expr>;

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'expr> {
    Var(usize),
    Lam(ExprRef<'expr>),
    App(ExprRef<'expr>, ExprRef<'expr>),
}

pub struct ExprBuilder<'expr> {
    arena: Arena<Expr<'expr>>,
}

impl<'expr> ExprBuilder<'expr> {
    pub fn new() -> Self {
        ExprBuilder {
            arena: Arena::new(),
        }
    }

    pub fn mk_app<'builder>(&'builder self, f: ExprRef<'expr>, x: ExprRef<'expr>) -> ExprRef<'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::App(f, x))
    }

    pub fn mk_lam<'builder>(&'builder self, x: ExprRef<'expr>) -> ExprRef<'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::Lam(x))
    }

    pub fn mk_var<'builder>(&'builder self, var: usize) -> ExprRef<'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::Var(var))
    }
}

#[test]
fn test_from_ast1() {
    let input = &syntax::Expr::Lam("x", &syntax::Expr::Ident("x"));
    let output = &Expr::Lam(&Expr::Var(0));
    let builder = ExprBuilder::new();
    assert_eq!(from_ast(&builder, input), output)
}

#[test]
fn test_from_ast2() {
    let input = &syntax::Expr::Lam("x", &syntax::Expr::Lam("y", &syntax::Expr::Ident("x")));
    let output = &Expr::Lam(&Expr::Lam(&Expr::Var(1)));
    let builder = ExprBuilder::new();
    assert_eq!(from_ast(&builder, input), output)
}

#[test]
fn test_from_ast3() {
    let input = &syntax::Expr::Lam("x", &syntax::Expr::Lam("y", &syntax::Expr::Ident("y")));
    let output = &Expr::Lam(&Expr::Lam(&Expr::Var(0)));
    let builder = ExprBuilder::new();
    assert_eq!(from_ast(&builder, input), output)
}

#[test]
fn test_from_ast4() {
    let input = &syntax::Expr::Lam(
        "x",
        &syntax::Expr::App(
            &syntax::Expr::Lam("x", &syntax::Expr::Ident("x")),
            &syntax::Expr::Ident("x"),
        ),
    );
    let output = &Expr::Lam(&Expr::App(&Expr::Lam(&Expr::Var(0)), &Expr::Var(0)));
    let builder = ExprBuilder::new();
    assert_eq!(from_ast(&builder, input), output)
}
