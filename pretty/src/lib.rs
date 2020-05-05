use ast::de_bruijn;
use ast::syntax;

pub fn pretty_syntax<'src, 'expr>(expr: syntax::ExprRef<'src, 'expr>) -> String {
    match expr {
        syntax::Expr::Ident(ident) => String::from(*ident),
        syntax::Expr::App(l, r) => {
            let parens_l = match &*l {
                syntax::Expr::Lam(_, _) => true,
                _ => false,
            };
            let parens_r = match &*r {
                syntax::Expr::Lam(_, _) => true,
                syntax::Expr::App(_, _) => true,
                _ => false,
            };
            let mut string = String::new();

            if parens_l {
                string.push('(');
            }
            string += &pretty_syntax(*l);
            if parens_l {
                string.push(')');
            }

            string.push(' ');

            if parens_r {
                string.push('(');
            }
            string += &pretty_syntax(*r);
            if parens_r {
                string.push(')');
            }

            string
        }
        syntax::Expr::Lam(arg, body) => {
            let mut string = String::from("\\");
            string += arg;
            string += " -> ";
            string += &pretty_syntax(*body);
            string
        }
        syntax::Expr::Parens(inner) => {
            let mut string = String::from("(");
            string += &pretty_syntax(*inner);
            string
        }
    }
}

pub fn pretty_de_bruijn<'expr>(expr: de_bruijn::ExprRef<'expr>) -> String {
    match expr {
        de_bruijn::Expr::Var(ix) => format!("#{}", ix),
        de_bruijn::Expr::U64(n) => format!("{}", n),
        de_bruijn::Expr::App(l, r) => {
            let parens_l = match &*l {
                de_bruijn::Expr::Lam(_) => true,
                _ => false,
            };
            let parens_r = match &*r {
                de_bruijn::Expr::Lam(_) => true,
                de_bruijn::Expr::App(_, _) => true,
                _ => false,
            };
            let mut string = String::new();

            if parens_l {
                string.push('(');
            }
            string += &pretty_de_bruijn(*l);
            if parens_l {
                string.push(')');
            }

            string.push(' ');

            if parens_r {
                string.push('(');
            }
            string += &pretty_de_bruijn(*r);
            if parens_r {
                string.push(')');
            }

            string
        }
        de_bruijn::Expr::AddU64(l, r) => {
            let parens_l = match &*l {
                de_bruijn::Expr::Lam(_) => true,
                _ => false,
            };
            let parens_r = match &*r {
                de_bruijn::Expr::Lam(_) => true,
                de_bruijn::Expr::AddU64(_, _) => true,
                _ => false,
            };
            let mut string = String::new();

            if parens_l {
                string.push('(');
            }
            string += &pretty_de_bruijn(*l);
            if parens_l {
                string.push(')');
            }

            string += " + ";

            if parens_r {
                string.push('(');
            }
            string += &pretty_de_bruijn(*r);
            if parens_r {
                string.push(')');
            }

            string
        }
        de_bruijn::Expr::Lam(body) => {
            let mut string = String::from("\\");
            string += ". ";
            string += &pretty_de_bruijn(*body);
            string
        }
    }
}
