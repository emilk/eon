//! Experimental `no_std` core for Eon.
//!
//! This crate is intentionally small:
//! - no external dependencies
//! - zero-copy parsing into borrowed events
//! - compact event-driven writing on top of `core::fmt::Write`
//!
//! The parser emits bare identifiers in value position as identifiers/symbols.
//! Higher layers can interpret those as unit enum variants, strings, or some
//! other typed construct depending on context.

#![no_std]
#![warn(missing_docs)]

mod error;
mod event;
mod parser;
mod span;
mod text;
mod writer;

pub use crate::{
    error::{Error, ErrorKind, Result},
    event::{Event, EventSink, Scalar, SpannedEvent, StringKind, StringToken, VariantName},
    parser::{DEFAULT_MAX_DEPTH, ParseError, parse, parse_with_limit},
    span::Span,
    text::{
        is_valid_identifier, write_escaped_string, write_scalar, write_symbol, write_variant_name,
    },
    writer::{EventWriter, SerializeError},
};

#[cfg(test)]
extern crate std;
