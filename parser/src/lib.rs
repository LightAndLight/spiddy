#[cfg(test)]
use ast::Expr;
use ast::{ExprBuilder, ExprRef};
use bit_set::BitSet;
use errors::Highlight;
use lazy_static::lazy_static;
#[cfg(test)]
use lexer::Lexer;
use lexer::{Token, TokenData, TokenType};
use span::Offset;
#[cfg(test)]
use span::{SourceFile, Span};
use std::fmt::{Debug, Display};
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq)]
pub enum Error<'src, 'tokens> {
    UnexpectedEof(Offset),
    Unexpected {
        actual: &'tokens Token<'src>,
        expected: ExpectedSet,
    },
}

impl<'src, 'tokens> Error<'src, 'tokens> {
    pub fn reportable(&self) -> errors::Error {
        match self {
            Error::UnexpectedEof(offset) => errors::Error {
                highlight: Highlight::Point(*offset),
                message: String::from("Unexpected end of input"),
            },

            Error::Unexpected { actual, expected } => errors::Error {
                highlight: Highlight::Span(actual.span),
                message: format!(
                    "Unexpected {}, expecting one of: {}",
                    actual.token_type(),
                    expected
                ),
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExpectedSet {
    bits: BitSet,
}

impl ExpectedSet {
    pub fn new() -> Self {
        ExpectedSet {
            bits: BitSet::with_capacity(1),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bits.clear();
    }

    #[inline]
    pub fn insert(&mut self, tt: &TokenType) {
        self.bits.insert(tt.to_usize());
    }

    #[inline]
    pub fn union(&mut self, other: &ExpectedSet) {
        self.bits.union_with(&other.bits);
    }

    #[inline]
    pub fn remove(&mut self, tt: &TokenType) {
        self.bits.remove(tt.to_usize());
    }

    #[inline]
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

impl Display for ExpectedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let vec = self.as_vec();
        let mut items = vec.iter();
        let () = match items.next() {
            Option::None => Result::Ok(()),
            Option::Some(item) => Display::fmt(item, formatter),
        }?;

        let mut result = Result::Ok(());
        for item in items {
            result?;
            formatter.write_str(", ")?;
            Display::fmt(item, formatter)?;
            result = Result::Ok(());
        }

        result
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
            )*
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
///        with_follows!(self, C_START_SET, self.b())
///        self.c() // follows inherited from parent
///    }
///    ```
///
///    'C' does accept the empty string:
///
///    ```ignore
///    fn a(&mut self) {
///        with_follows_extended!(self, C_START_SET, self.b())
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
///        with_follows!(self, {'++', '--'}, self.b())
///    }
///    ```
///
/// If this were function then the utility of an explicit `follows` stack would be wasted.
/// The call stack would be duplicating the work of the `follows` stack.
#[macro_export]
macro_rules! with_follows {
    ($self:ident, $followed_by:expr, $cont:block) => {{
        $self.follows.push($followed_by);
        let res = $cont;
        $self.follows.pop();
        res
    }};
}

/// Like `with_follows`, but uses the most recent follow set as a base. See `with_follows` for usage.
#[macro_export]
macro_rules! with_follows_extended {
    ( $self:ident, $followed_by:expr, $cont:block ) => {{
        let last_follows = match $self.follows.last() {
            Option::None => $followed_by.clone(),
            Option::Some(last_follows) => {
                let mut last_follows = last_follows.clone();
                last_follows.union($followed_by);
                last_follows
            }
        };
        with_follows!($self, last_follows, $cont)
    }};
}

pub type ParseResult<'src, 'tokens, T> = Result<T, Error<'src, 'tokens>>;

pub struct Parser<'src, 'tokens, 'builder, 'expr> {
    builder: &'builder ExprBuilder<'src, 'expr>,
    current: Option<&'tokens Token<'src>>,
    position: Iter<'tokens, Token<'src>>,
    expected: ExpectedSet,
    follows: Vec<ExpectedSet>,
}

lazy_static! {
    static ref EXPECTED_RPAREN: ExpectedSet = expected![&TokenType::RParen];
    static ref ATOM_START_SET: ExpectedSet = expected![&TokenType::Ident, &TokenType::LParen];
}

impl<'src, 'tokens, 'builder, 'expr> Parser<'src, 'tokens, 'builder, 'expr> {
    /// `input` must be terminated by a `TokenType::Eof`
    pub fn new(
        builder: &'builder ExprBuilder<'src, 'expr>,
        input: &'tokens Vec<Token<'src>>,
    ) -> Self {
        let expected = ExpectedSet::new();
        let follows = Vec::new();
        let mut position = input.iter();
        let current = position.next();

        Parser {
            builder,
            current,
            position,
            expected,
            follows,
        }
    }

    #[inline]
    fn current_token(&self) -> &'tokens Token<'src> {
        match self.current {
            Option::Some(token) => token,
            Option::None => panic!("current_token failed: ran out of input"),
        }
    }

    #[inline]
    fn consume(&mut self) -> Option<&'tokens Token<'src>> {
        let res = self.position.next();
        self.current = res;
        res
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

    fn unexpected_with<T>(&self, extra: &ExpectedSet) -> ParseResult<'src, 'tokens, T> {
        let actual = self.current_token();
        let mut expected = self.expected.clone();
        expected.union(extra);
        Result::Err(Error::Unexpected { actual, expected })
    }

    #[inline]
    fn unexpected<T>(&mut self) -> ParseResult<'src, 'tokens, T> {
        self.unexpected_with(&ExpectedSet::new())
    }

    fn expect_ident(&mut self) -> Option<&'src str> {
        self.expect(&TokenType::Ident)
            .and_then(|token| match token.data {
                TokenData::Ident(ident) => Option::Some(ident),
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
        while let TokenData::Space | TokenData::Newline = self.current_token().data {
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
    fn try_parse_atom(&mut self) -> ParseResult<'src, 'tokens, Option<ExprRef<'src, 'expr>>>
    where
        'builder: 'expr,
    {
        match self.expect_ident() {
            Option::Some(ident) => {
                self.ignore_spaces();
                Result::Ok(Option::Some(self.builder.mk_ident(ident)))
            }
            Option::None => match self.expect(&TokenType::LParen) {
                Option::Some(_) => {
                    self.ignore_spaces();

                    let inner =
                        with_follows!(self, (*EXPECTED_RPAREN).clone(), { self.parse_expr() })?;

                    let _ = self.require(&TokenType::RParen)?;
                    let _ = self.ignore_spaces();

                    Result::Ok(Option::Some(self.builder.mk_parens(inner)))
                }
                Option::None => Result::Ok(Option::None),
            },
        }
    }

    /// ```ignore
    /// lambda ::=
    ///   '\' ident '->' expr
    /// ```
    fn try_parse_lam(&mut self) -> ParseResult<'src, 'tokens, Option<ExprRef<'src, 'expr>>>
    where
        'builder: 'expr,
    {
        match self.expect(&TokenType::Backslash) {
            Option::Some(_) => {
                let _ = self.ignore_spaces();

                let arg = self.require_ident()?;
                let _ = self.ignore_spaces();

                let _ = self.require(&TokenType::RArrow)?;
                let _ = self.ignore_spaces();

                let body = self.parse_expr()?;

                Result::Ok(Option::Some(self.builder.mk_lam(arg, body)))
            }
            Option::None => Result::Ok(Option::None),
        }
    }

    /// ```ignore
    /// app ::=
    ///   atom atom*
    /// ```
    fn try_parse_app(&mut self) -> ParseResult<'src, 'tokens, Option<ExprRef<'src, 'expr>>>
    where
        'builder: 'expr,
    {
        let atom_res = with_follows_extended!(self, &*ATOM_START_SET, { self.try_parse_atom() })?;
        match atom_res {
            Option::Some(head) => {
                let mut result = head;
                loop {
                    let atom_res =
                        with_follows_extended!(self, &*ATOM_START_SET, { self.try_parse_atom() });
                    match atom_res {
                        Result::Err(err) => return Result::Err(err),
                        Result::Ok(Option::None) => {
                            let token = self.current_token();
                            match self.follows.last() {
                                Option::None => {
                                    return self.unexpected_with(&ExpectedSet::new());
                                }
                                Option::Some(followed_by) => {
                                    if followed_by.contains(&token.token_type()) {
                                        break;
                                    } else {
                                        return self.unexpected_with(&followed_by);
                                    }
                                }
                            }
                        }
                        Result::Ok(Option::Some(expr)) => {
                            result = self.builder.mk_app(result, expr);
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
    fn parse_expr(&mut self) -> ParseResult<'src, 'tokens, ExprRef<'src, 'expr>>
    where
        'builder: 'expr,
    {
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

    pub fn parse_expr_eof(&mut self) -> ParseResult<'src, 'tokens, ExprRef<'src, 'expr>>
    where
        'builder: 'expr,
    {
        with_follows!(self, expected![&TokenType::Eof], { self.parse_expr() })
    }
}

#[cfg(test)]
fn test_parser<'src, 'expr>(input: String, expected: ExprRef<'src, 'expr>) {
    let source_file = SourceFile {
        name: String::from("test"),
        start: Offset(0),
        content: input,
    };
    let lexer_res = Lexer::from_source_file(&source_file).tokenize();
    match lexer_res {
        Result::Ok(ref tokens) => {
            let builder = ExprBuilder::new();
            assert_eq!(
                Parser::new(&builder, tokens).parse_expr_eof(),
                Result::Ok(expected)
            )
        }
        Result::Err(err) => panic!(format!("{:?}", err)),
    }
}

#[cfg(test)]
fn test_parser_fail<'src, 'tokens>(input: String, expected: Error<'src, 'tokens>) {
    let source_file = SourceFile {
        name: String::from("test"),
        start: Offset(0),
        content: input,
    };
    let lexer_res = Lexer::from_source_file(&source_file).tokenize();
    match lexer_res {
        Result::Ok(ref tokens) => {
            let builder = ExprBuilder::new();
            assert_eq!(
                Parser::new(&builder, tokens).parse_expr_eof(),
                Result::Err(expected)
            )
        }
        Result::Err(err) => panic!(format!("{:?}", err)),
    }
}

#[test]
fn test_parser_ident() {
    let input = String::from("hello");
    test_parser(input, &Expr::Ident("hello"))
}

#[test]
fn test_parser_lambda() {
    let input = String::from("\\x -> x");
    test_parser(input, &Expr::Lam("x", &Expr::Ident("x")))
}

#[test]
fn test_parser_app_2() {
    let input = String::from("x x");
    test_parser(input, &Expr::App(&Expr::Ident("x"), &Expr::Ident("x")))
}

#[test]
fn test_parser_app_4() {
    let input = String::from("what is love baby");

    let builder = ExprBuilder::new();
    let expected = builder.mk_apps(
        builder.mk_ident("what"),
        vec![
            builder.mk_ident("is"),
            builder.mk_ident("love"),
            builder.mk_ident("baby"),
        ],
    );
    test_parser(input, expected)
}

#[test]
fn test_parser_app_fail1() {
    let input = String::from("x \\y -> y");
    test_parser_fail(
        input,
        Error::Unexpected {
            actual: &Token {
                data: TokenData::Backslash,
                span: Span {
                    start: Offset(2),
                    length: Offset(1),
                },
            },
            expected: expected![&TokenType::Ident, &TokenType::LParen, &TokenType::Eof],
        },
    )
}

#[test]
fn test_parser_app_fail2() {
    let input = String::from("(x \\y -> y)");
    test_parser_fail(
        input,
        Error::Unexpected {
            actual: &Token {
                data: TokenData::Backslash,
                span: Span {
                    start: Offset(3),
                    length: Offset(1),
                },
            },
            expected: expected![&TokenType::Ident, &TokenType::LParen, &TokenType::RParen],
        },
    );
}

#[test]
fn test_parser_app_fail3() {
    let input = String::from("x y \\z -> z");
    test_parser_fail(
        input,
        Error::Unexpected {
            actual: &Token {
                data: TokenData::Backslash,
                span: Span {
                    start: Offset(4),
                    length: Offset(1),
                },
            },
            expected: expected![&TokenType::Ident, &TokenType::LParen, &TokenType::Eof],
        },
    );
}

#[test]
fn test_parser_app_fail4() {
    let input = String::from("(x y \\z -> z)");
    test_parser_fail(
        input,
        Error::Unexpected {
            actual: &Token {
                data: TokenData::Backslash,
                span: Span {
                    start: Offset(5),
                    length: Offset(1),
                },
            },
            expected: expected![&TokenType::Ident, &TokenType::LParen, &TokenType::RParen],
        },
    );
}

#[test]
fn test_parser_parens() {
    let input = String::from("(x)");
    test_parser(input, &Expr::Parens(&Expr::Ident("x")))
}
