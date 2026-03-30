//! Zero-dependency formatter input model for Eon.
//!
//! This crate preserves a borrowed stream of tokens and trivia so a formatter
//! can later reason about comments, whitespace, and line breaks without
//! depending on `logos`, `ariadne`, or the richer syntax tree.

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod error;
mod lexer;
mod span;
mod token;

pub use crate::{
    error::{Error, ErrorKind, Result},
    lexer::lex,
    span::Span,
    token::{Item, StringKind, Token, TokenKind, TokenStream, Trivia, TriviaKind},
};

#[cfg(test)]
extern crate std;
