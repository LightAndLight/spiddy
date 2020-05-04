use errors::Highlight;
use span::{Offset, SourceFile, Span};
use std::convert::TryInto;
use std::fmt::Display;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Space,
    Newline,
    Backslash,
    Ident,
    RArrow,
    LParen,
    RParen,
    Equals,
    Eof,
}

impl Display for TokenType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str(match self {
            TokenType::Space => "' '",
            TokenType::Newline => "newline",
            TokenType::Backslash => "'\\'",
            TokenType::Ident => "identifier",
            TokenType::RArrow => "'->'",
            TokenType::LParen => "'('",
            TokenType::RParen => "')'",
            TokenType::Equals => "'='",
            TokenType::Eof => "end of input",
        })
    }
}

impl TokenType {
    pub fn to_usize(&self) -> usize {
        match self {
            TokenType::Space => 0,
            TokenType::Newline => 1,
            TokenType::Backslash => 2,
            TokenType::Ident => 3,
            TokenType::RArrow => 4,
            TokenType::LParen => 5,
            TokenType::RParen => 6,
            TokenType::Equals => 7,
            TokenType::Eof => 8,
        }
    }

    pub fn unsafe_from_usize(i: usize) -> Self {
        match i {
            0 => TokenType::Space,
            1 => TokenType::Newline,
            2 => TokenType::Backslash,
            3 => TokenType::Ident,
            4 => TokenType::RArrow,
            5 => TokenType::LParen,
            6 => TokenType::RParen,
            7 => TokenType::Equals,
            8 => TokenType::Eof,
            _ => panic!("unsafe_from_usize failed"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenData<'src> {
    Space,
    Newline,
    Backslash,
    Ident(&'src str),
    RArrow,
    LParen,
    RParen,
    Equals,
    Eof,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'src> {
    pub data: TokenData<'src>,
    pub span: Span,
}

impl<'src> Token<'src> {
    #[inline]
    pub fn token_type(&self) -> TokenType {
        match self.data {
            TokenData::Space => TokenType::Space,
            TokenData::Newline => TokenType::Newline,
            TokenData::Backslash => TokenType::Backslash,
            TokenData::Ident(_) => TokenType::Ident,
            TokenData::RArrow => TokenType::RArrow,
            TokenData::LParen => TokenType::LParen,
            TokenData::RParen => TokenType::RParen,
            TokenData::Equals => TokenType::Equals,
            TokenData::Eof => TokenType::Eof,
        }
    }
}

pub struct Lexer<'src> {
    src_file: &'src SourceFile,
    current: Option<char>,
    position: Chars<'src>,
    /// offset in bytes; *not* characters (we assume UTF-8 encoding)
    offset: Offset,
}

#[inline]
fn is_ident_start(c: &char) -> bool {
    c.is_ascii_lowercase()
}

#[inline]
fn is_ident_body(c: &char) -> bool {
    c.is_ascii_alphanumeric()
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unexpected(char, Offset),
    UnexpectedEof(Offset),
}

impl Error {
    pub fn reportable(&self) -> errors::Error {
        match self {
            Error::Unexpected(c, offset) => errors::Error {
                highlight: Highlight::Point(*offset),
                message: format!("Unexpected symbol '{}'", c),
            },
            Error::UnexpectedEof(offset) => errors::Error {
                highlight: Highlight::Point(*offset),
                message: String::from("Unexpected end of input"),
            },
        }
    }
}

pub type LexerResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum NextToken<'src> {
    Done,
    Token(Token<'src>),
    Error(Error),
}

impl<'src> Lexer<'src> {
    pub fn from_source_file(src_file: &'src SourceFile) -> Self {
        let mut position = src_file.data().chars();
        let current = position.next();
        Lexer {
            src_file,
            current,
            position,
            offset: src_file.get_start(),
        }
    }

    #[inline]
    fn lookahead(&mut self) -> Option<char> {
        self.current
    }

    fn consume(&mut self) {
        let m_c = self.current;
        self.offset
            .add_mut(m_c.map_or(0, |c| c.len_utf8().try_into().unwrap()));
        self.current = self.position.next();
    }

    fn consume_ident_body(&mut self, start_offset: Offset) -> Token<'src> {
        while let Some(ref c) = self.lookahead() {
            if !is_ident_body(c) {
                break;
            }
            self.consume();
        }
        let end_offset = self.offset;
        let data =
            TokenData::Ident(&self.src_file.data()[start_offset.to_usize()..end_offset.to_usize()]);
        let span = Span {
            start: start_offset,
            length: end_offset.subtract(start_offset.to_u32()),
        };
        Token { data, span }
    }

    fn unexpected(&self, c: char) -> Error {
        Error::Unexpected(c, self.offset)
    }

    fn unexpected_eof(&self) -> Error {
        Error::UnexpectedEof(self.offset)
    }

    fn emit(&mut self, start_offset: Offset, data: TokenData<'src>) -> NextToken<'src> {
        self.consume();
        let end_offset = self.offset;
        let span = Span {
            start: start_offset,
            length: end_offset.subtract(start_offset.to_u32()),
        };
        NextToken::Token(Token { data, span })
    }

    fn next_token(&mut self) -> NextToken<'src> {
        let start_offset = self.offset;
        match self.lookahead() {
            Option::None => NextToken::Done,
            Option::Some(c) => match c {
                '\n' => self.emit(start_offset, TokenData::Newline),
                ' ' => self.emit(start_offset, TokenData::Space),
                '\\' => self.emit(start_offset, TokenData::Backslash),
                '-' =>
                // RArrow
                {
                    self.consume();
                    match self.lookahead() {
                        Option::Some('>') => self.emit(start_offset, TokenData::RArrow),
                        Option::Some(c) => NextToken::Error(self.unexpected(c)),
                        Option::None => NextToken::Error(self.unexpected_eof()),
                    }
                }
                '(' => self.emit(start_offset, TokenData::LParen),
                ')' => self.emit(start_offset, TokenData::RParen),
                '=' => self.emit(start_offset, TokenData::Equals),
                _ if is_ident_start(&c) => {
                    self.consume();
                    NextToken::Token(self.consume_ident_body(start_offset))
                }
                _ => NextToken::Error(self.unexpected(c)),
            },
        }
    }

    pub fn tokenize(mut self) -> LexerResult<Vec<Token<'src>>> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                NextToken::Done => {
                    let offset = self.offset;
                    tokens.push(Token {
                        data: TokenData::Eof,
                        span: Span {
                            start: offset,
                            length: Offset(1),
                        },
                    });
                    break;
                }
                NextToken::Token(token) => {
                    tokens.push(token);
                }
                NextToken::Error(err) => {
                    return Result::Err(err);
                }
            }
        }
        Result::Ok(tokens)
    }
}

#[cfg(test)]
fn test_source_file(content: String) -> SourceFile {
    SourceFile {
        name: String::from("test"),
        start: Offset(0),
        content,
    }
}

#[test]
fn test_lexer_example1() {
    let src_file = test_source_file(String::from("->"));
    assert_eq!(
        Lexer::from_source_file(&src_file).next_token(),
        NextToken::Token(Token {
            data: TokenData::RArrow,
            span: Span {
                start: Offset(0),
                length: Offset(2)
            }
        })
    );
}

#[test]
fn test_lexer_example2() {
    let src_file = test_source_file(String::from("hello"));
    assert_eq!(
        Lexer::from_source_file(&src_file).next_token(),
        NextToken::Token(Token {
            data: TokenData::Ident("hello"),
            span: Span {
                start: Offset(0),
                length: Offset(5)
            }
        })
    );
}

#[test]
fn test_lexer_example3() {
    let src_file = test_source_file(String::from("f = \\input -> input"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Ok(vec![
            Token {
                data: TokenData::Ident("f"),
                span: Span {
                    start: Offset(0),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(1),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Equals,
                span: Span {
                    start: Offset(2),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(3),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Backslash,
                span: Span {
                    start: Offset(4),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Ident("input"),
                span: Span {
                    start: Offset(5),
                    length: Offset(5)
                }
            },
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(10),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::RArrow,
                span: Span {
                    start: Offset(11),
                    length: Offset(2)
                }
            },
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(13),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Ident("input"),
                span: Span {
                    start: Offset(14),
                    length: Offset(5)
                }
            },
            Token {
                data: TokenData::Eof,
                span: Span {
                    start: Offset(19),
                    length: Offset(1)
                }
            },
        ])
    );
}

#[test]
fn test_lexer_example4() {
    let src_file = test_source_file(String::from("  aa"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Err(Error::Unexpected('', Offset(4)))
    );
}

#[test]
fn test_lexer_example5() {
    let src_file = test_source_file(String::from("  aa\naa"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Ok(vec![
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(0),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Space,
                span: Span {
                    start: Offset(1),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Ident("aa"),
                span: Span {
                    start: Offset(2),
                    length: Offset(2)
                }
            },
            Token {
                data: TokenData::Newline,
                span: Span {
                    start: Offset(4),
                    length: Offset(1)
                }
            },
            Token {
                data: TokenData::Ident("aa"),
                span: Span {
                    start: Offset(5),
                    length: Offset(2)
                }
            },
            Token {
                data: TokenData::Eof,
                span: Span {
                    start: Offset(7),
                    length: Offset(1)
                }
            },
        ])
    );
}

#[test]
fn test_lexer_example6() {
    let src_file = test_source_file(String::from("  aa\na"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Err(Error::Unexpected('', Offset(6)))
    );
}
