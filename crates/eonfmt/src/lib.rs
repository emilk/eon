//! Minimal formatter library for Eon.
//!
//! This crate re-exports the reusable formatter surface from
//! [`eon_formatter_core`] so consumers can depend on `eonfmt` without pulling
//! in the CLI-only dependencies. Disable the default `cli` feature when using
//! it as a library-only dependency.

#![warn(missing_docs)]

pub use eon_formatter_core::*;

/// Format an Eon string using the default formatting options.
pub fn format_str(source: &str) -> Result<String> {
    reformat(source, &FormatOptions::default())
}

/// Return whether the provided Eon source is already formatted according to the
/// provided options.
pub fn is_formatted(source: &str, options: &FormatOptions) -> Result<bool> {
    Ok(reformat(source, options)? == source)
}
