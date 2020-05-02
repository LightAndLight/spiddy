use std::iter::Peekable;
use std::str::CharIndices;
use std::string::String;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Space,
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
            TokenType::Backslash => 1,
            TokenType::Ident => 2,
            TokenType::RArrow => 3,
            TokenType::LParen => 4,
            TokenType::RParen => 5,
            TokenType::Equals => 6,
            TokenType::Eof => 7,
        }
    }

    pub fn unsafe_from_usize(i: usize) -> Self {
        match i {
            0 => TokenType::Space,
            1 => TokenType::Backslash,
            2 => TokenType::Ident,
            3 => TokenType::RArrow,
            4 => TokenType::LParen,
            5 => TokenType::RParen,
            6 => TokenType::Equals,
            7 => TokenType::Eof,
            _ => panic!("unsafe_from_usize failed"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'src> {
    Space,
    Backslash,
    Ident(&'src str),
    RArrow,
    LParen,
    RParen,
    Equals,
    Eof,
}

impl<'src> Token<'src> {
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Space => TokenType::Space,
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
    input: &'src str,
    position: Peekable<CharIndices<'src>>,
}

fn is_ident_start(c: &char) -> bool {
    c.is_ascii_lowercase()
}

fn is_ident_body(c: &char) -> bool {
    c.is_ascii_alphanumeric()
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
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unexpected(char),
    UnexpectedEof,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NextToken<'src> {
    Done,
    Token(Token<'src>),
    Error(Error),
}

impl<'src> Lexer<'src> {
    pub fn from_string(input: &'src String) -> Self {
        let position = input.char_indices().peekable();
        Lexer {
            input: input.as_str(),
            position,
        }
    }

    fn consume_ident_body(&mut self, start_pos: usize) -> Token<'src> {
        let mut end_pos: usize = start_pos + 1;
        while let Some((_, c)) = self.position.peek() {
            if is_ident_body(c) {
                self.position.next();
                end_pos += 1;
            } else {
                break;
            };
        }
        Token::Ident(&self.input[start_pos..end_pos])
    }

    fn next_token(&mut self) -> NextToken<'src> {
        match self.position.next() {
            Option::None => NextToken::Done,
            Option::Some((ix, c)) => match c {
                ' ' => NextToken::Token(Token::Space),
                '\\' => NextToken::Token(Token::Backslash),
                '-' =>
                // RArrow
                {
                    match self.position.next() {
                        Option::Some((_, '>')) => NextToken::Token(Token::RArrow),
                        Option::Some((_, c)) => NextToken::Error(Error::Unexpected(c)),
                        Option::None => NextToken::Error(Error::UnexpectedEof),
                    }
                }
                '(' => NextToken::Token(Token::LParen),
                ')' => NextToken::Token(Token::RParen),
                '=' => NextToken::Token(Token::Equals),
                _ if is_ident_start(&c) => NextToken::Token(self.consume_ident_body(ix)),
                _ => NextToken::Error(Error::Unexpected(c)),
            },
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token<'src>>, Error> {
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
