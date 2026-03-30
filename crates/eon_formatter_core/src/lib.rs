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
mod format;
mod lexer;
mod parse;
mod span;
mod syntax;
mod token;
mod view;

pub use crate::{
    document::{Document as RootDocument, DocumentKind, MapEntry, ValueSpan, analyze_document},
    error::{Error, ErrorKind, ParseErrorKind, Result},
    format::{FormatOptions, reformat},
    lexer::lex,
    parse::{DEFAULT_MAX_DEPTH, parse_document},
    span::Span,
    syntax::{Comment, Document, KeyValue, List, Map, Value, ValueTree, Variant, VariantName},
    token::{Item, StringKind, Token, TokenKind, TokenStream, Trivia, TriviaKind},
    view::{LeadingTriviaKind, TokenRef, TokenRefs, TriviaIter},
};

#[cfg(test)]
extern crate std;
