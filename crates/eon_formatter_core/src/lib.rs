//! Zero-dependency formatter input model for Eon.
//!
//! This crate preserves a borrowed stream of tokens and trivia so a formatter
//! can later reason about comments, whitespace, and line breaks without
//! depending on `logos`, `ariadne`, or the richer syntax tree.

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod document;
mod error;
mod lexer;
mod span;
mod token;
mod view;

pub use crate::{
    document::{Document, DocumentKind, MapEntry, ValueSpan, analyze_document},
    error::{Error, ErrorKind, Result},
    lexer::lex,
    span::Span,
    token::{Item, StringKind, Token, TokenKind, TokenStream, Trivia, TriviaKind},
    view::{LeadingTriviaKind, TokenRef, TokenRefs, TriviaIter},
};

#[cfg(test)]
extern crate std;
