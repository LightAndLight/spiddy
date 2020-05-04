use typed_arena::Arena;

pub type ExprRef<'src, 'expr> = &'expr Expr<'src, 'expr>;

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'src, 'expr> {
    Ident(&'src str),
    Lam(&'src str, ExprRef<'src, 'expr>),
    App(ExprRef<'src, 'expr>, ExprRef<'src, 'expr>),
    Parens(ExprRef<'src, 'expr>),
}

pub struct ExprBuilder<'src, 'expr> {
    arena: Arena<Expr<'src, 'expr>>,
}

impl<'src, 'expr> ExprBuilder<'src, 'expr> {
    pub fn new() -> Self {
        ExprBuilder {
            arena: Arena::new(),
        }
    }

    pub fn mk_app<'builder>(
        &'builder self,
        f: ExprRef<'src, 'expr>,
        x: ExprRef<'src, 'expr>,
    ) -> ExprRef<'src, 'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::App(f, x))
    }

    pub fn mk_apps<'builder>(
        &'builder self,
        f: ExprRef<'src, 'expr>,
        xs: Vec<ExprRef<'src, 'expr>>,
    ) -> ExprRef<'src, 'expr>
    where
        'builder: 'expr,
    {
        let mut expr = f;
        for x in xs.iter() {
            expr = self.arena.alloc(Expr::App(expr, x))
        }
        expr
    }

    pub fn mk_lam<'builder>(
        &'builder self,
        arg: &'src str,
        x: ExprRef<'src, 'expr>,
    ) -> ExprRef<'src, 'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::Lam(arg, x))
    }

    pub fn mk_parens<'builder>(&'builder self, inner: ExprRef<'src, 'expr>) -> ExprRef<'src, 'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::Parens(inner))
    }

    pub fn mk_ident<'builder>(&'builder self, ident: &'src str) -> ExprRef<'src, 'expr>
    where
        'builder: 'expr,
    {
        self.arena.alloc(Expr::Ident(ident))
    }
}
