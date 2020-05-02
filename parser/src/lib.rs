use ast::Expr;
use bit_set::BitSet;
use lexer::{Lexer, Token, TokenType};
use std::fmt::Debug;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'src, 'tokens> {
    UnexpectedEof,
    Unexpected {
        actual: &'tokens Token<'src>,
        expected: Vec<TokenType>,
    },
}

#[derive(Clone)]
pub struct ExpectedSet {
    bits: BitSet,
}

impl ExpectedSet {
    pub fn new() -> Self {
        ExpectedSet {
            bits: BitSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.bits.clear();
    }

    pub fn insert(&mut self, tt: &TokenType) {
        self.bits.insert(tt.to_usize());
    }

    pub fn union(&mut self, other: &ExpectedSet) {
        self.bits.union_with(&other.bits);
    }

    pub fn remove(&mut self, tt: &TokenType) {
        self.bits.remove(tt.to_usize());
    }

    pub fn contains(&self, tt: &TokenType) -> bool {
        self.bits.contains(tt.to_usize())
    }

    pub fn as_vec(&self) -> Vec<TokenType> {
        self.bits
            .iter()
            .map(|i| TokenType::unsafe_from_usize(i))
            .collect()
    }
}

impl Debug for ExpectedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.as_vec().fmt(formatter)
    }
}

#[macro_export]
macro_rules! expected {
    [ $( $tt:expr ),* ] => {
        {
            let mut followed_by = ExpectedSet::new();
            $(
                followed_by.insert($tt);
            );*
            followed_by
        }
    }
}

/// If a non-terminal is followed by a set of terminal symbols, then run it in the context of `with_follows`
/// to make those terminal symbols available for diagnostics.
///
/// Examples:
///
/// 1.
///
///    ```ignore
///    A ::=
///      B C
///    ```
///
///    'C' doesn't accept the empty string:
///
///    ```ignore
///    fn a(&mut self) {
///        self.with_follows(C_START_SET, |this| this.b())
///        self.c() // follows inherited from parent
///    }
///    ```
///
///    'C' does accept the empty string:
///
///    ```ignore
///    fn a(&mut self) {
///        self.with_follows_extended(C_START_SET, |this| this.b())
///        self.c() // follows inherited from parent
///    }
///    ```
///
/// 2.
///
///    ```ignore
///    A ::=
///      B '++'
///      B '--'
///    ```
///
///    ```ignore
///    fn a(&mut self) {
///        self.with_follows({'++', '--'}, |this| this.b())
///    }
///    ```
///
/// If this were function then the utility of an explicit `follows` stack would be wasted.
/// The call stack would be duplicating the work of the `follows` stack.
#[macro_export]
macro_rules! with_follows {
    ($self:ident, $followed_by:expr, $cont:expr) => {{
        $self.follows.push($followed_by);
        let res = $cont($self);
        $self.follows.pop();
        res
    }};
}

/// Like `with_follows`, but uses the most recent follow set as a base. See `with_follows` for usage.
#[macro_export]
macro_rules! with_follows_extended {
    ( $self:ident, $followed_by:expr, $cont:expr ) => {{
        let mut last_follows = $self
            .follows
            .last()
            .map_or_else(|| ExpectedSet::new(), |set| set.clone());
        last_follows.union(&$followed_by);
        with_follows!($self, last_follows, $cont)
    }};
}

pub type ParseResult<'src, 'tokens, T> = Result<T, ParseError<'src, 'tokens>>;

pub struct Parser<'src, 'tokens> {
    position: Peekable<Iter<'tokens, Token<'src>>>,
    expected: ExpectedSet,
    follows: Vec<ExpectedSet>,
    atom_start_set: ExpectedSet,
}

impl<'src, 'tokens> Parser<'src, 'tokens> {
    pub fn new(input: &'tokens Vec<Token<'src>>) -> Self {
        let expected = ExpectedSet::new();
        let follows = Vec::new();
        let atom_start_set = expected![&TokenType::Ident, &TokenType::LParen];

        Parser {
            position: input.iter().peekable(),
            expected,
            follows,
            atom_start_set,
        }
    }

    #[inline]
    fn current_token(&mut self) -> &'tokens Token<'src> {
        match self.position.peek() {
            Option::Some(token) => token,
            Option::None => &Token::Eof,
        }
    }

    #[inline]
    fn consume(&mut self) -> Option<&'tokens Token<'src>> {
        self.position.next()
    }

    fn expect(&mut self, tt: &'tokens TokenType) -> Option<&'tokens Token<'src>> {
        self.expected.insert(tt);
        let token = self.current_token();
        if token.token_type() == *tt {
            match self.consume() {
                Option::Some(_) => {
                    self.expected.clear();
                }
                Option::None => (),
            }
            Option::Some(token)
        } else {
            Option::None
        }
    }

    fn unexpected_with<T>(&mut self, extra: &ExpectedSet) -> ParseResult<'src, 'tokens, T> {
        let actual = self.current_token();
        self.expected.union(extra);
        let expected = self.expected.as_vec();
        Result::Err(ParseError::Unexpected { actual, expected })
    }

    #[inline]
    fn unexpected<T>(&mut self) -> ParseResult<'src, 'tokens, T> {
        self.unexpected_with(&ExpectedSet::new())
    }

    fn expect_ident(&mut self) -> Option<&'src str> {
        self.expect(&TokenType::Ident)
            .and_then(|token| match *token {
                Token::Ident(ident) => Option::Some(ident),
                _ => Option::None,
            })
    }

    fn require(
        &mut self,
        tt: &'tokens TokenType,
    ) -> ParseResult<'src, 'tokens, &'tokens Token<'src>> {
        match self.expect(tt) {
            Option::Some(token) => Result::Ok(token),
            Option::None => self.unexpected(),
        }
    }

    fn require_ident(&mut self) -> ParseResult<'src, 'tokens, &'src str> {
        match self.expect_ident() {
            Option::Some(ident) => Result::Ok(ident),
            Option::None => self.unexpected(),
        }
    }

    fn ignore_spaces(&mut self) -> usize {
        let mut count = 0;
        while let Token::Space = self.current_token() {
            let _ = self.consume();
            count += 1;
        }
        count
    }

    /// ```ignore
    /// atom ::=
    ///   ident
    ///   '(' expr ')'
    /// ```
    fn try_parse_atom(&mut self) -> ParseResult<'src, 'tokens, Option<Expr<'src>>> {
        match self.expect_ident() {
            Option::Some(ident) => {
                self.ignore_spaces();
                Result::Ok(Option::Some(Expr::Ident(ident)))
            }
            Option::None => match self.expect(&TokenType::LParen) {
                Option::Some(_) => {
                    self.ignore_spaces();

                    let inner =
                        with_follows!(self, expected![&TokenType::RParen], |this: &mut Self| this
                            .parse_expr())?;

                    let _ = self.require(&TokenType::RParen)?;

                    Result::Ok(Option::Some(Expr::Parens(Box::new(inner))))
                }
                Option::None => Result::Ok(Option::None),
            },
        }
    }

    /// ```ignore
    /// lambda ::=
    ///   '\' ident '->' expr
    /// ```
    fn try_parse_lam(&mut self) -> ParseResult<'src, 'tokens, Option<Expr<'src>>> {
        match self.expect(&TokenType::Backslash) {
            Option::Some(_) => {
                let _ = self.ignore_spaces();

                let arg = self.require_ident()?;
                let _ = self.ignore_spaces();

                let _ = self.require(&TokenType::RArrow)?;
                let _ = self.ignore_spaces();

                let body = self.parse_expr()?;

                Result::Ok(Option::Some(Expr::Lam(arg, Box::new(body))))
            }
            Option::None => Result::Ok(Option::None),
        }
    }

    /// ```ignore
    /// app ::=
    ///   atom atom*
    /// ```
    fn try_parse_app(&mut self) -> ParseResult<'src, 'tokens, Option<Expr<'src>>> {
        let atom_res =
            with_follows_extended!(self, self.atom_start_set.clone(), |this: &mut Self| this
                .try_parse_atom())?;
        match atom_res {
            Option::Some(head) => {
                let mut result = head;
                loop {
                    let atom_res = with_follows_extended!(
                        self,
                        self.atom_start_set.clone(),
                        |this: &mut Self| this.try_parse_atom()
                    );
                    match atom_res {
                        Result::Err(err) => return Result::Err(err),
                        Result::Ok(Option::None) => {
                            let token = self.current_token();
                            let followed_by = self
                                .follows
                                .last()
                                .map_or(ExpectedSet::new(), |followed_by| followed_by.clone());
                            if followed_by.contains(&token.token_type()) {
                                break;
                            }
                            return self.unexpected_with(&followed_by);
                        }
                        Result::Ok(Option::Some(expr)) => {
                            result = Expr::mk_app(result, expr);
                        }
                    }
                }
                Result::Ok(Option::Some(result))
            }
            Option::None => Result::Ok(Option::None),
        }
    }

    /// ```ignore
    /// expr ::=
    ///   lambda
    ///   app
    /// ```
    fn parse_expr(&mut self) -> ParseResult<'src, 'tokens, Expr<'src>> {
        let lam_result = self.try_parse_lam()?;
        match lam_result {
            Option::Some(expr) => Result::Ok(expr),
            Option::None => {
                let app_result = self.try_parse_app()?;
                match app_result {
                    Option::Some(expr) => Result::Ok(expr),
                    Option::None => self.unexpected(),
                }
            }
        }
    }

    fn parse_expr_eof(&mut self) -> ParseResult<'src, 'tokens, Expr<'src>> {
        let mut followed_by = ExpectedSet::new();
        followed_by.insert(&TokenType::Eof);
        self.follows.push(followed_by);
        let res = self.parse_expr();
        self.follows.pop();
        res
    }
}

#[cfg(test)]
fn test_parser<'src>(input: &'src String, expected: Expr<'src>) {
    let lexer_res = Lexer::from_string(input).tokenize();
    match lexer_res {
        Result::Ok(ref tokens) => {
            assert_eq!(Parser::new(tokens).parse_expr_eof(), Result::Ok(expected))
        }
        Result::Err(err) => panic!(format!("{:?}", err)),
    }
}

#[cfg(test)]
fn test_parser_fail<'src, 'tokens>(input: &'src String, expected: ParseError<'src, 'tokens>) {
    let lexer_res = Lexer::from_string(input).tokenize();
    match lexer_res {
        Result::Ok(ref tokens) => {
            assert_eq!(Parser::new(tokens).parse_expr_eof(), Result::Err(expected))
        }
        Result::Err(err) => panic!(format!("{:?}", err)),
    }
}

#[test]
fn test_parser_ident() {
    let input = String::from("hello");
    test_parser(&input, Expr::Ident("hello"))
}

#[test]
fn test_parser_lambda() {
    let input = String::from("\\x -> x");
    test_parser(&input, Expr::Lam("x", Box::new(Expr::Ident("x"))))
}

#[test]
fn test_parser_app_2() {
    let input = String::from("x x");
    test_parser(
        &input,
        Expr::App(Box::new(Expr::Ident("x")), Box::new(Expr::Ident("x"))),
    )
}

#[test]
fn test_parser_app_4() {
    let input = String::from("what is love baby");
    let head = Expr::Ident("what");
    let tail = vec!["is", "love", "baby"];
    test_parser(
        &input,
        tail.into_iter().fold(head, |acc, x| {
            Expr::App(Box::new(acc), Box::new(Expr::Ident(x)))
        }),
    )
}

#[test]
fn test_parser_app_fail1() {
    let input = String::from("x \\y -> y");
    test_parser_fail(
        &input,
        ParseError::Unexpected {
            actual: &Token::Backslash,
            expected: vec![TokenType::Ident, TokenType::LParen, TokenType::Eof],
        },
    )
}

#[test]
fn test_parser_app_fail2() {
    let input = String::from("(x \\y -> y)");
    test_parser_fail(
        &input,
        ParseError::Unexpected {
            actual: &Token::Backslash,
            expected: vec![TokenType::Ident, TokenType::LParen, TokenType::RParen],
        },
    );
}

#[test]
fn test_parser_app_fail3() {
    let input = String::from("x y \\z -> z");
    test_parser_fail(
        &input,
        ParseError::Unexpected {
            actual: &Token::Backslash,
            expected: vec![TokenType::Ident, TokenType::LParen, TokenType::Eof],
        },
    );
}

#[test]
fn test_parser_app_fail4() {
    let input = String::from("(x y \\z -> z)");
    test_parser_fail(
        &input,
        ParseError::Unexpected {
            actual: &Token::Backslash,
            expected: vec![TokenType::Ident, TokenType::LParen, TokenType::RParen],
        },
    );
}

#[test]
fn test_parser_parens() {
    let input = String::from("(x)");
    test_parser(&input, Expr::Parens(Box::new(Expr::Ident("x"))))
}
