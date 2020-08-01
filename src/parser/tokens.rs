use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use super::span::{Span, Position};
use crate::parser::{Parser, State};

#[derive(Debug)]
pub enum TokenKind {
    Text(String),
    LeftBracket,
    RightBracket,
    Equal,
    Semicolon,
    Argument(char),
}

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.kind {
            TokenKind::Text(data) => f.write_fmt("\"{}\"", data),
            TokenKind::LeftBracket => f.write_char('['),
            TokenKind::RightBracket => f.write_char(']'),
            TokenKind::Equal => f.write_char('='),
            TokenKind::Semicolon => f.write_char(';'),
            TokenKind::Argument(a) => f.write_fmt("%{}", a),
        }

        Ok(())
    }
}

impl Token {
    pub fn new(kind: TokenKind, pos: Position) -> Token {
        Token {
            kind,
            span: Span::new(pos, pos + 1),
        }
    }

    pub fn new_with_span(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }

    pub fn is_text(&self) -> bool {
        match self.kind {
            TokenKind::Text(..) => true,
            _ => false,
        }
    }

    pub fn is_left_bracket(&self) -> bool {
        match self.kind {
            TokenKind::LeftBracket => true,
            _ => false,
        }
    }

    pub fn is_right_bracket(&self) -> bool {
        match self.kind {
            TokenKind::RightBracket => true,
            _ => false,
        }
    }

    pub fn is_equal(&self) -> bool {
        match self.kind {
            TokenKind::Equal => true,
            _ => false,
        }
    }

    pub fn is_semicolon(&self) -> bool {
        match self.kind {
            TokenKind::Semicolon => true,
            _ => false,
        }
    }

    pub fn is_argument(&self) -> bool {
        match self.kind {
            TokenKind::Argument(..) => true,
            _ => false,
        }
    }
}

pub struct TokenData<'a> {
    pub data: Peekable<Chars<'a>>,
    pub pos: Position,
    back_data: Vec<char>,
}

impl<'a> TokenData<'a> {
    pub fn new(data: impl Into<&'a str>) -> TokenData<'a> {
        let src = data.into();

        TokenData {
            data: src.chars().peekable(),
            pos: Position::new(),
            back_data: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = if self.back_data.len() > 0 {
            self.back_data.pop()
        } else {
            self.data.next()
        };

        if c.is_some() {
            self.pos += 1;
        }

        c
    }

    fn peek(&mut self) -> Option<&char> {
        if self.back_data.len() > 0 {
            return Some(&self.back_data[self.back_data.len() - 1]);
        }

        self.data.peek()
    }

    fn push(&mut self, c: char) {
        self.pos -= 1;
        self.back_data.push(c);
    }
}

pub struct Tokens<'a> {
    data: TokenData<'a>,
    state: State,
    buf: String,
}

impl<'a> Tokens<'a> {
    pub fn new(data: impl Into<&'a str>) -> Tokens<'a>
    {
        Tokens {
            data: TokenData::new(data),
            state: State::ReadKey,
            buf: String::with_capacity(2048),
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        loop {
            match self.data.peek() {
                Some(' ') | Some('\t') if self.state != State::ReadExec => self.skip_whitespace(),
                Some('#') => self.skip_comment(),
                Some('\n') | Some('\r') => self.advance_line(),
                Some('[') if self.state == State::ReadKey => {
                    self.state = State::ReadHeader;

                    self.data.next();
                    return Some(Token::new(TokenKind::LeftBracket, self.data.pos));
                },
                Some(']') if self.state == State::ReadHeader => {
                    self.state = State::ReadKey;

                    self.data.next();
                    return Some(Token::new(TokenKind::RightBracket, self.data.pos));
                },
                Some('=') if self.state == State::ReadKey => {
                    self.state = State::ReadValue;
                    self.state = if self.buf == "Exec" {
                        State::ReadExec
                    } else {
                        State::ReadValue
                    };

                    self.data.next();
                    return Some(Token::new(TokenKind::Equal, self.data.pos));
                },
                Some(';') if self.state == State::ReadValue => {
                    self.data.next();
                    return Some(Token::new(TokenKind::Semicolon, self.data.pos));
                },
                Some('%') if self.state == State::ReadExec => {
                    self.data.next();

                    if let Some(arg) = self.data.next() {
                        return Some(Token::new(TokenKind::Argument(arg), self.data.pos));
                    } else {
                        return Some(Token::new(TokenKind::Argument(0 as char), self.data.pos));
                    }
                }
                None => return None,
                _ => {
                    let start = self.data.pos;

                    self.read_text();
                    let span = Span::new(start, self.data.pos);
                    let kind = TokenKind::Text(self.buf.clone());
                    return Some(Token::new_with_span(kind, span));
                },
            };
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.data.next() {
                Some(' ') | Some('\t') => continue,
                None => return,
                Some(c) => {
                    self.data.push(c);
                    return;
                },
            }
        }
    }

    fn skip_comment(&mut self) {
        loop {
            match self.data.next() {
                Some(c) if c == '\n' || c == '\r' => {
                    self.data.push(c);
                    self.advance_line();
                    return;
                },
                None => return,
                _ => continue,
            }
        }
    }

    fn advance_line(&mut self) {
        match self.data.next() {
            Some('\n') => (),
            Some('\r') => {
                match self.data.next() {
                    Some('\n') => (),
                    Some(c) => self.data.push(c),
                    None => (),
                }
            },
            Some(c) => {
                self.data.push(c);
                return;
            },
            None => return,
        }

        self.data.pos.newline();
        self.state = State::ReadKey;
    }

    fn read_text(&mut self) {
        self.buf.clear();

        let mut test_arg = false;
        loop {
            if test_arg {
                test_arg = false;

                self.data.next();
                let arg = self.data.peek();
                match arg {
                    Some(c) if c.is_alphabetic() => {
                        self.data.push('%');
                        break;
                    },
                    _ => self.buf.push('%'),
                }
            }

            match self.data.peek() {
                Some('[') if self.state == State::ReadKey => break,
                Some(']') if self.state == State::ReadKey => break,
                Some(']') if self.state == State::ReadHeader => break,
                Some('=') if self.state == State::ReadKey => break,
                Some(';') if self.state == State::ReadValue => break,
                Some('\n') | Some('\r') | Some('#') => break,
                Some('%') if self.state == State::ReadExec => {
                    test_arg = true;
                    continue;
                },
                Some(n) => self.buf.push(n.clone()),
                None => return,
            }

            self.data.next();
        }
    }

}

impl Iterator for Tokens<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text() {
        let p = Parser::new("abc]=\n");

        let mut t = p.tokens();
        t.state = State::ReadHeader;
        t.read_text();
        assert_eq!(t.buf, "abc");

        let mut t = p.tokens();
        t.state = State::ReadValue;
        t.read_text();
        assert_eq!(t.buf, "abc]=");

        let p = Parser::new("abc%f=\n");
        
        let mut t = p.tokens();
        t.state = State::ReadKey;
        t.read_text();
        assert_eq!(t.buf, "abc%f");

        let mut t = p.tokens();
        t.state = State::ReadExec;
        t.read_text();
        assert_eq!(t.buf, "abc");

        let mut t = p.tokens();
        t.state = State::ReadValue;
        t.read_text();
        assert_eq!(t.buf, "abc%f=");

        let mut t = p.tokens();
        t.state = State::ReadHeader;
        t.read_text();
        assert_eq!(t.buf, "abc%f=");

        let mut t = Tokens::new("value #comment");
        t.state = State::ReadValue;
        t.read_text();
        assert_eq!(t.buf, "value ");

        let mut t = Tokens::new("text1;text2");
        t.state = State::ReadValue;
        t.read_text();
        assert_eq!(t.buf, "text1");

        let mut t = Tokens::new("text1\n");
        t.state = State::ReadValue;
        t.read_text();
        assert_eq!(t.buf, "text1");
    }

    #[test]
    fn test_advance_line() {
        let mut t = Tokens::new("\n\n\r\n\r");
        t.state = State::ReadValue;
        
        assert_eq!(t.data.pos.row, 0);
        assert_eq!(t.data.pos.col, 0);
        assert_eq!(t.data.pos.idx, 0);

        t.advance_line();
        assert_eq!(t.data.pos.row, 1);
        assert_eq!(t.data.pos.col, 0);
        assert_eq!(t.data.pos.idx, 1);


        t.advance_line();
        assert_eq!(t.data.pos.row, 2);
        assert_eq!(t.data.pos.col, 0);
        assert_eq!(t.data.pos.idx, 2);

        t.advance_line();
        assert_eq!(t.data.pos.row, 3);
        assert_eq!(t.data.pos.col, 0);
        assert_eq!(t.data.pos.idx, 4);

        t.advance_line();
        assert_eq!(t.data.pos.row, 4);
        assert_eq!(t.data.pos.col, 0);
        assert_eq!(t.data.pos.idx, 5);
    }

    #[test]
    fn test_skip_comment() {
        let mut t = Tokens::new("text #comment!\r\nmore text");
        t.state = State::ReadValue;

        t.read_text();
        assert_eq!(t.buf, "text ");

        t.skip_comment();
        assert_eq!(t.data.pos.row, 1);
        assert_eq!(t.data.pos.col, 0);

        t.read_text();
        assert_eq!(t.buf, "more text");
    }

    #[test]
    fn test_skip_whitespace() {
        let mut t = Tokens::new("\t\t     text\r\n   \t\t");
        t.state = State::ReadValue;

        t.skip_whitespace();
        assert_eq!(t.data.pos.row, 0);
        assert_eq!(t.data.pos.col, 7);

        t.read_text();
        assert_eq!(t.buf, "text");

        t.advance_line();
        assert_eq!(t.data.pos.row, 1);
        assert_eq!(t.data.pos.col, 0);

        t.skip_whitespace();
        assert_eq!(t.data.pos.row, 1);
        assert_eq!(t.data.pos.col, 5);
    }

    #[test]
    fn test_next_token() {
        let mut t = Tokens::new(r#"
        [header]
        key1[en]=Hello World! [text] = stuff #this is a comment
        key2=./hello %F lol
        Exec=/usr/bin/app %f --arg %%
        #comment on a line
        key3=list;of;stuff!
        "#);

        assert_eq!(t.state, State::ReadKey);
        assert!(t.next_token().unwrap().is_left_bracket());
        assert_eq!(t.state, State::ReadHeader);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "header");
        }

        assert!(t.next_token().unwrap().is_right_bracket());
        assert_eq!(t.state, State::ReadKey);
        
        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "key1");
        }

        assert_eq!(t.state, State::ReadKey);
        assert!(t.next_token().unwrap().is_left_bracket());
        
        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "en");
        }

        assert!(t.next_token().unwrap().is_right_bracket());
        assert!(t.next_token().unwrap().is_equal());
        assert_eq!(t.state, State::ReadValue);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "Hello World! [text] = stuff ");
        }

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "key2")
        }
        
        assert_eq!(t.state, State::ReadKey);
        assert!(t.next_token().unwrap().is_equal());
        assert_eq!(t.state, State::ReadValue);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "./hello %F lol");
        }

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "Exec");
        }

        assert_eq!(t.state, State::ReadKey);
        assert!(t.next_token().unwrap().is_equal());
        assert_eq!(t.state, State::ReadExec);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "/usr/bin/app ");
        }

        assert_eq!(t.state, State::ReadExec);
        
        let tok = t.next_token().unwrap();
        assert!(tok.is_argument());
        if let TokenKind::Argument(c) = tok.kind {
            assert_eq!(c, 'f');
        }

        assert_eq!(t.state, State::ReadExec);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, " --arg %%");
        }

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "key3");
        }

        assert_eq!(t.state, State::ReadKey);
        assert!(t.next_token().unwrap().is_equal());
        assert_eq!(t.state, State::ReadValue);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "list");
        }

        assert_eq!(t.state, State::ReadValue);
        assert!(t.next_token().unwrap().is_semicolon());

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "of");
        }
        assert!(t.next_token().unwrap().is_semicolon());

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "stuff!");
        }

        let tok = t.next_token();
        assert!(tok.is_none());
    }

    #[test]
    fn test_token_multiple_skips() {
        let mut t = Tokens::new("key=value\r\t \t#a comment\n\n    #another comment\tyes\n\t    key2=\tvalue");

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "key");
        }

        assert_eq!(tok.span.start.col, 0);
        assert_eq!(tok.span.end.col, 3);

        assert!(t.next_token().unwrap().is_equal());

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "value");
        }

        assert_eq!(tok.span.start.col, 4);
        assert_eq!(tok.span.end.col, 9);

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "key2");
        }

        assert_eq!(tok.span.start.row, 4);
        assert_eq!(tok.span.start.col, 5);
        assert_eq!(tok.span.end.col, 9);

        assert!(t.next_token().unwrap().is_equal());

        let tok = t.next_token().unwrap();
        assert!(tok.is_text());
        if let TokenKind::Text(val) = tok.kind {
            assert_eq!(val, "value");
        }

        assert_eq!(tok.span.start.col, 11);
        assert_eq!(tok.span.end.col, 16);
    }
}