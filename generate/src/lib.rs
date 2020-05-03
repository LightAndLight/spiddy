use ast::Expr;

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

pub struct Generator {
    idents: Vec<String>,
}

impl Generator {
    pub fn new() -> Self {
        let mut idents = Vec::new();
        let alphabet: Vec<char> = ALPHABET.chars().collect();
        for _ in 0..100 {
            let length = (rand::random::<u8>() % 10) + 1;
            let mut ident = String::new();
            for _ in 0..length {
                ident.push(alphabet[rand::random::<usize>() % 26])
            }
            idents.push(ident);
        }
        Generator { idents }
    }

    fn gen_ident<'gen>(&'gen self) -> &'gen str {
        let existing_count = self.idents.len();
        &self.idents[rand::random::<usize>() % existing_count]
    }

    pub fn gen_expr<'gen>(&'gen self, size: u32) -> Expr<'gen> {
        if size > 0 {
            match rand::random::<u8>() % 2 {
                0 => self.gen_lambda(size),
                1 => self.gen_app(size),
                _ => panic!("impossible"),
            }
        } else {
            Expr::Ident(&self.gen_ident())
        }
    }

    fn gen_app<'gen>(&'gen self, size: u32) -> Expr<'gen> {
        let l = self.gen_expr(size - 1);
        let r = self.gen_expr(size - 1);
        Expr::mk_app(l, r)
    }

    fn gen_lambda<'gen>(&'gen self, size: u32) -> Expr<'gen> {
        let arg = &self.gen_ident();
        let body = self.gen_expr(size - 1);
        Expr::mk_lam(arg, body)
    }
}
