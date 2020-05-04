use ast::de_bruijn::ExprRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'expr, 'value> {
    Closure {
        env: Vec<&'value Value<'expr, 'value>>,
        body: ExprRef<'expr>,
    },
}
