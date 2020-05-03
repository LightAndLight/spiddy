use ast::Expr;

pub fn pretty_expr(expr: Expr) -> String {
    match expr {
        Expr::Ident(ident) => String::from(ident),
        Expr::App(l, r) => {
            let parens_l = match &*l {
                Expr::Lam(_, _) => true,
                _ => false,
            };
            let parens_r = match &*r {
                Expr::Lam(_, _) => true,
                Expr::App(_, _) => true,
                _ => false,
            };
            let mut string = String::new();

            if parens_l {
                string.push('(');
            }
            string += &pretty_expr(*l);
            if parens_l {
                string.push(')');
            }

            string.push(' ');

            if parens_r {
                string.push('(');
            }
            string += &pretty_expr(*r);
            if parens_r {
                string.push(')');
            }

            string
        }
        Expr::Lam(arg, body) => {
            let mut string = String::from("\\");
            string += arg;
            string += " -> ";
            string += &pretty_expr(*body);
            string
        }
        Expr::Parens(inner) => {
            let mut string = String::from("(");
            string += &pretty_expr(*inner);
            string
        }
    }
}
