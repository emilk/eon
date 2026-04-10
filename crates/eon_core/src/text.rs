use core::fmt;

use crate::{Scalar, StringToken, VariantName};

fn is_keyword(string: &str) -> bool {
    matches!(string, "true" | "false" | "null")
}

/// Returns `true` if the string matches `[a-zA-Z_][a-zA-Z0-9_]*` and is not a keyword.
pub fn is_valid_identifier(string: &str) -> bool {
    if is_keyword(string) {
        return false;
    }

    let mut chars = string.chars();
    if chars
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '_')
    {
        return false;
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Write a string with double quotes and minimal escaping.
pub fn write_escaped_string<W>(writer: &mut W, raw: &str) -> fmt::Result
where
    W: fmt::Write,
{
    writer.write_char('"')?;
    for chr in raw.chars() {
        match chr {
            '"' => writer.write_str("\\\"")?,
            '\\' => writer.write_str("\\\\")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            chr if chr.is_control() || should_escape_for_visibility(chr) => {
                write!(writer, "\\u{{{:X}}}", chr as u32)?
            }
            chr => writer.write_char(chr)?,
        }
    }
    writer.write_char('"')
}

fn should_escape_for_visibility(chr: char) -> bool {
    !matches!(chr, '"' | '\\' | '\'' | '\n' | '\r' | '\t')
        && chr.escape_debug().next() == Some('\\')
}

/// Write a bare symbol when possible, otherwise fall back to a quoted string.
pub fn write_symbol<W>(writer: &mut W, symbol: &str) -> fmt::Result
where
    W: fmt::Write,
{
    if is_valid_identifier(symbol) {
        writer.write_str(symbol)
    } else {
        write_escaped_string(writer, symbol)
    }
}

/// Write a scalar token in its canonical compact form.
pub fn write_scalar<W>(writer: &mut W, scalar: Scalar<'_>) -> fmt::Result
where
    W: fmt::Write,
{
    match scalar {
        Scalar::Null => writer.write_str("null"),
        Scalar::Bool(true) => writer.write_str("true"),
        Scalar::Bool(false) => writer.write_str("false"),
        Scalar::Number(number) => writer.write_str(number),
        Scalar::Identifier(identifier) => write_symbol(writer, identifier),
        Scalar::String(StringToken { raw, .. }) => writer.write_str(raw),
    }
}

/// Write a variant name in its compact form.
pub fn write_variant_name<W>(writer: &mut W, name: VariantName<'_>) -> fmt::Result
where
    W: fmt::Write,
{
    match name {
        VariantName::Identifier(identifier) => write_symbol(writer, identifier),
        VariantName::String(StringToken { raw, .. }) => writer.write_str(raw),
    }
}
