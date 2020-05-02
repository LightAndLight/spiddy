use std::iter::Peekable;
use std::str::Chars;
use std::string::String;

#[derive(Debug, PartialEq, Eq)]
pub struct Pos<'src> {
    filename: Option<&'src str>,
    line: usize,
    column: usize,
}

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
    filename: Option<&'src str>,
    input: &'src str,
    position: Peekable<Chars<'src>>,
    offset: usize,
    line: usize,
    column: usize,
}

#[inline]
fn is_newline(c: char) -> bool {
    match c {
        '\n' => true,
        _ => false,
    }
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
pub enum Error<'src> {
    Unexpected(char, Pos<'src>),
    UnexpectedEof(Pos<'src>),
}

pub type LexerResult<'src, T> = Result<T, Error<'src>>;

#[derive(Debug, PartialEq, Eq)]
pub enum NextToken<'src> {
    Done,
    Token(Token<'src>),
    Error(Error<'src>),
}

impl<'src> Lexer<'src> {
    pub fn from_string(input: &'src String) -> Self {
        let position = input.chars().peekable();
        Lexer {
            filename: Option::None,
            input: input.as_str(),
            position,
            offset: 0,
            line: 0,
            column: 0,
        }
    }

    #[inline]
    pub fn set_filename(&mut self, filename: &'src String) {
        self.filename = Option::Some(filename)
    }

    #[inline]
    fn get_pos(&self) -> Pos<'src> {
        Pos {
            filename: self.filename,
            line: self.line,
            column: self.column,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.position.next().map(|c| {
            self.offset += 1;
            if is_newline(c) {
                self.line += 1;
                self.column = 0
            } else {
                self.column += 1
            };
            c
        })
    }

    fn consume_ident_body(&mut self, start_offset: usize) -> Token<'src> {
        while let Some(c) = self.position.peek() {
            if !is_ident_body(c) {
                break;
            }
            self.next_char();
        }
        let end_offset = self.offset;
        Token::Ident(&self.input[start_offset..end_offset])
    }

    fn unexpected(&self, c: char) -> Error<'src> {
        Error::Unexpected(c, self.get_pos())
    }

    fn unexpected_eof(&self) -> Error<'src> {
        Error::UnexpectedEof(self.get_pos())
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

    pub fn tokenize(mut self) -> LexerResult<'src, Vec<Token<'src>>> {
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

#[test]
fn test_lexer_example1() {
    assert_eq!(
        Lexer::from_string(&String::from("->")).next_token(),
        NextToken::Token(Token::RArrow)
    );
}

#[test]
fn test_lexer_example2() {
    assert_eq!(
        Lexer::from_string(&String::from("hello")).next_token(),
        NextToken::Token(Token::Ident("hello"))
    );
}

#[test]
fn test_lexer_example3() {
    let input = String::from("f = \\input -> input");
    let lexer = Lexer::from_string(&input);
    assert_eq!(
        lexer.tokenize(),
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
    let input = String::from("  aa");
    let lexer = Lexer::from_string(&input);
    assert_eq!(
        lexer.tokenize(),
        Result::Err(Error::Unexpected(
            '',
            Pos {
                filename: Option::None,
                line: 0,
                column: 5
            }
        ))
    );
}

#[test]
fn test_lexer_example5() {
    let input = String::from("  aa\naa");
    let lexer = Lexer::from_string(&input);
    assert_eq!(
        lexer.tokenize(),
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
    let input = String::from("  aa\na");
    let lexer = Lexer::from_string(&input);
    assert_eq!(
        lexer.tokenize(),
        Result::Err(Error::Unexpected(
            '',
            Pos {
                filename: Option::None,
                line: 1,
                column: 2
            }
        ))
    );
}
