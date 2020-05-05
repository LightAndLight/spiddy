use ast::de_bruijn::ExprRef;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'expr, 'value> {
    U64(u64),
    Closure {
        env: Rc<Vec<&'value Value<'expr, 'value>>>,
        body: ExprRef<'expr>,
    },
}
