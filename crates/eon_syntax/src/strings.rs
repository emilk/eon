use crate::Result;

/// Returns `true` if the string does NOT match `[a-zA-Z_][a-zA-Z0-9_]*`
pub fn key_needs_quotes(string: &str) -> bool {
    if matches!(string, "true" | "false" | "null") {
        return true; // We need to quote these to avoid confusion with the special keyword values
    }

    let mut chars = string.chars();

    if chars
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '_')
    {
        return true;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return true;
        }
    }

    false
}

#[test]
fn test_needs_quotes() {
    assert!(key_needs_quotes(""));
    assert!(key_needs_quotes("true"));
    assert!(!key_needs_quotes("True"));
    assert!(!key_needs_quotes("a"));
    assert!(!key_needs_quotes("a1"));
    assert!(!key_needs_quotes("a_b"));
    assert!(!key_needs_quotes("a_b1"));
    assert!(key_needs_quotes("1a"));
    assert!(!key_needs_quotes("_1a"));
    assert!(key_needs_quotes("a-b"));
    assert!(key_needs_quotes("a b"));
}

pub fn escape_and_quote(raw: &str) -> String {
    #![expect(clippy::unwrap_used)] // Can't fail - the Debug format always produces a string with quotes.

    let escaped = format!("{raw:?}");

    if raw.contains('"') && !raw.contains('\'') {
        // Use single-quotes instead of double-quotes.
        let unquoted = escaped
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap(); // Remove quotes
        let unquoted = unquoted.replace("\\\"", "\""); // No need to escape double-quotes
        format!("'{unquoted}'") // Use single-quotes
    } else {
        escaped
    }
}

/// Remove the quotes and unescape the string.
pub fn unescape_and_unquote(escaped: &str) -> Result<String, snailquote::UnescapeError> {
    debug_assert!(
        escaped.starts_with('"') || escaped.starts_with('\''),
        "Expected a quoted string, got: {escaped}"
    );
    snailquote::unescape(escaped)
}

#[test]
fn test_escape() {
    assert_eq!(escape_and_quote("normal"), r#""normal""#);
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes'"#),
        r#""a string with 'single-quotes'""#
    );
    assert_eq!(
        escape_and_quote(r#"a string with "double-quotes""#),
        r#"'a string with "double-quotes"'"#
    );
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes' and "double-quotes""#),
        r#""a string with 'single-quotes' and \"double-quotes\"""#
    );
    assert_eq!(
        escape_and_quote("a string with newline \n"),
        r#""a string with newline \n""#
    );
}
