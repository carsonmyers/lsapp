pub mod error;
pub mod span;
pub mod tokens;
pub mod tree;

use color_eyre::{Report, Result};
use std::iter::Peekable;
use std::str::Chars;

use tokens::Tokens;

#[derive(PartialEq, Debug)]
pub enum State {
    ReadHeader,
    ReadKey,
    ReadValue,
    ReadExec,
}

pub struct Parser<'a> {
    data: TokenData<'a>,
    tokens: Option<tokens::Tokens>,
    fwd: Vec<tokens::Token>,
}

impl Parser {
    pub fn new(data: impl Into<String>) -> Parser {
        Parser { data: TokenData::new(data) }
    }

    pub fn parse(&mut self) -> Result<Vec<Section>> {

    }

    fn match_heading(&mut self) -> Result<Section> {

    }

    fn match_entry(&mut self) -> Result<Entry> {
       self.data.match_token()
    }

    fn match_text(&mut self, tok tokens::Token) -> Option<tokens::Token> {
        
    }

    fn next(&mut self) -> Option<tokens::Token> {
        if len(self.fwd) == 0 {
            
        }
    }
}