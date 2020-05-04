use ast::de_bruijn::ExprRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'expr, 'value> {
    U64(u64),
    Closure {
        env: Vec<&'value Value<'expr, 'value>>,
        body: ExprRef<'expr>,
    },
}
