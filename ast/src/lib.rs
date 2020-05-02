#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'src> {
    Ident(&'src str),
    Lam(&'src str, Box<Expr<'src>>),
    App(Box<Expr<'src>>, Box<Expr<'src>>),
    Parens(Box<Expr<'src>>),
}

impl<'src> Expr<'src> {
    pub fn mk_app(f: Self, x: Self) -> Self {
        Expr::App(Box::new(f), Box::new(x))
    }
}
