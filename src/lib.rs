#[allow(unused_imports)]
#[macro_use]
extern crate relexer_derive;

#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;
pub use lazy_static::*;


#[macro_use]
pub extern crate failure;
pub extern crate regex;

#[doc(hidden)]
pub use relexer_derive::*;

use std::marker::PhantomData;

pub trait Token: Sized {
    fn produce<'a>(input: &'a str) -> (Result<Self>, &'a str);
    fn skip(&self) -> bool;
}

pub struct TokenIterator<'input, T: Token> {
    stopped : bool,
    input: &'input str,
    phantom: PhantomData<T>,
}
impl<'input, T: Token> TokenIterator<'input, T> {
    pub fn into_inner(self) -> &'input str {
        self.input
    }
}
impl<'input, T: Token> Iterator for TokenIterator<'input, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        'main: loop {
            if self.stopped || self.input.is_empty() {
                self.stopped = true;
                return None;
            }
            let (tok, input) = T::produce(self.input);
            if let Ok(ref t) = tok {
                self.input = input;
                if t.skip() {
                    continue 'main;
                }
            } else {
                self.stopped = true;
            }
            return Some(tok)
        }
    }
}

pub fn scan<T: Token>(input: &str) -> TokenIterator<T> {
    TokenIterator {
        input,
        stopped: false,
        phantom: PhantomData
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to parse token of type {} with regex {}: {}",ty,regex,parsed )]
    InvalidToken {
        parsed: String,
        regex: &'static str,
        ty: &'static str,
    },
    #[fail(display = "No rule in the lexer matches: {}", unparsed)]
    InvalidInput {
        unparsed: String
    }
}
pub type Result<T> = std::result::Result<T, Error>;
