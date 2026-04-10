use std::borrow::Cow;

use eon_core::{StringToken, VariantName};

pub(crate) fn decode_string_token(token: StringToken<'_>) -> Result<Cow<'_, str>, String> {
    if let Some(decoded) = token.decoded_if_borrowed() {
        Ok(Cow::Borrowed(decoded))
    } else {
        eon_syntax::unescape_and_unquote(token.raw).map(Cow::Owned)
    }
}

pub(crate) fn decode_variant_name(name: VariantName<'_>) -> Result<Cow<'_, str>, String> {
    match name {
        VariantName::Identifier(identifier) => Ok(Cow::Borrowed(identifier)),
        VariantName::String(token) => decode_string_token(token),
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use eon_core::StringKind;

    use super::{decode_string_token, decode_variant_name};

    #[test]
    fn decode_string_token_borrows_literal_strings() {
        let decoded = decode_string_token(eon_core::StringToken {
            raw: "'hello'",
            kind: StringKind::Literal,
        })
        .unwrap();

        assert!(matches!(decoded, Cow::Borrowed("hello")));
    }

    #[test]
    fn decode_string_token_owns_escaped_basic_strings() {
        let decoded = decode_string_token(eon_core::StringToken {
            raw: "\"he\\nllo\"",
            kind: StringKind::Basic,
        })
        .unwrap();

        assert!(matches!(decoded, Cow::Owned(ref value) if value == "he\nllo"));
    }

    #[test]
    fn decode_variant_name_borrows_identifier_variants() {
        let decoded = decode_variant_name(eon_core::VariantName::Identifier("EnumValue")).unwrap();

        assert!(matches!(decoded, Cow::Borrowed("EnumValue")));
    }
}
