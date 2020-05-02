use span::{Offset, SourceFile};
use std::convert::TryInto;
use std::iter::Peekable;
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
pub enum Token<'src> {
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

impl<'src> Token<'src> {
    #[inline]
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Space => TokenType::Space,
            Token::Newline => TokenType::Newline,
            Token::Backslash => TokenType::Backslash,
            Token::Ident(_) => TokenType::Ident,
            Token::RArrow => TokenType::RArrow,
            Token::LParen => TokenType::LParen,
            Token::RParen => TokenType::RParen,
            Token::Equals => TokenType::Equals,
            Token::Eof => TokenType::Eof,
        }
    }
}

pub struct Lexer<'src> {
    src_file: &'src SourceFile,
    position: Peekable<Chars<'src>>,
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

pub type LexerResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum NextToken<'src> {
    Done,
    Token(Token<'src>),
    Error(Error),
}

impl<'src> Lexer<'src> {
    pub fn from_source_file(src_file: &'src SourceFile) -> Self {
        let position = src_file.data().chars().peekable();
        Lexer {
            src_file,
            position,
            offset: src_file.get_start(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.position.next().map(|c| {
            self.offset.add(c.len_utf8().try_into().unwrap());
            c
        })
    }

    fn consume_ident_body(&mut self, start_offset: Offset) -> Token<'src> {
        while let Some(c) = self.position.peek() {
            if !is_ident_body(c) {
                break;
            }
            self.next_char();
        }
        let end_offset = self.offset;
        Token::Ident(&self.src_file.data()[start_offset.to_usize()..end_offset.to_usize()])
    }

    fn unexpected(&self, c: char) -> Error {
        Error::Unexpected(c, self.offset)
    }

    fn unexpected_eof(&self) -> Error {
        Error::UnexpectedEof(self.offset)
    }

    fn next_token(&mut self) -> NextToken<'src> {
        let start_offset = self.offset;
        match self.next_char() {
            Option::None => NextToken::Done,
            Option::Some(c) => match c {
                '\n' => NextToken::Token(Token::Newline),
                ' ' => NextToken::Token(Token::Space),
                '\\' => NextToken::Token(Token::Backslash),
                '-' =>
                // RArrow
                {
                    match self.next_char() {
                        Option::Some('>') => NextToken::Token(Token::RArrow),
                        Option::Some(c) => NextToken::Error(self.unexpected(c)),
                        Option::None => NextToken::Error(self.unexpected_eof()),
                    }
                }
                '(' => NextToken::Token(Token::LParen),
                ')' => NextToken::Token(Token::RParen),
                '=' => NextToken::Token(Token::Equals),
                _ if is_ident_start(&c) => NextToken::Token(self.consume_ident_body(start_offset)),
                _ => NextToken::Error(self.unexpected(c)),
            },
        }
    }

    pub fn tokenize(mut self) -> LexerResult<Vec<Token<'src>>> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                NextToken::Done => {
                    tokens.push(Token::Eof);
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
        NextToken::Token(Token::RArrow)
    );
}

#[test]
fn test_lexer_example2() {
    let src_file = test_source_file(String::from("hello"));
    assert_eq!(
        Lexer::from_source_file(&src_file).next_token(),
        NextToken::Token(Token::Ident("hello"))
    );
}

#[test]
fn test_lexer_example3() {
    let src_file = test_source_file(String::from("f = \\input -> input"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Ok(vec![
            Token::Ident("f"),
            Token::Space,
            Token::Equals,
            Token::Space,
            Token::Backslash,
            Token::Ident("input"),
            Token::Space,
            Token::RArrow,
            Token::Space,
            Token::Ident("input"),
            Token::Eof
        ])
    );
}

#[test]
fn test_lexer_example4() {
    let src_file = test_source_file(String::from("  aa"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Err(Error::Unexpected('', Offset(5)))
    );
}

#[test]
fn test_lexer_example5() {
    let src_file = test_source_file(String::from("  aa\naa"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Ok(vec![
            Token::Space,
            Token::Space,
            Token::Ident("aa"),
            Token::Newline,
            Token::Ident("aa"),
            Token::Eof
        ])
    );
}

#[test]
fn test_lexer_example6() {
    let src_file = test_source_file(String::from("  aa\na"));
    assert_eq!(
        Lexer::from_source_file(&src_file).tokenize(),
        Result::Err(Error::Unexpected('', Offset(7)))
    );
}
