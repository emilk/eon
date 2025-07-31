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

/// Format a string into Eon, adding quotes and escaping as needed.
///
/// The exact type of quoting (single, double, multiline basic, or multiline literal) will be determined automatically based on the content of the string.
// TODO: smartly choose between all four types of strings: double, single, multiline basic, and multiline literal.
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
pub fn unescape_and_unquote(escaped: &str) -> Result<String, String> {
    if let Some(suffix) = escaped.strip_prefix("'''") {
        // multiline literal string. No escape sequences, but strip the leading newline (if any):
        let Some(contents) = suffix.strip_suffix("'''") else {
            return Err(
                "Triple-quoted multiline literal string must end with three single quotes"
                    .to_owned(),
            );
        };
        if let Some(stripped_newline) = contents.strip_prefix('\n') {
            Ok(stripped_newline.to_owned())
        } else {
            Ok(contents.to_owned())
        }
    } else if let Some(suffix) = escaped.strip_prefix("'") {
        // single-quoted literal string. No escape sequences.
        let Some(contents) = suffix.strip_suffix("'") else {
            return Err(
                "Single-quoted literal string must end with three single quotes".to_owned(),
            );
        };
        if contents.contains('\n') {
            Err("Single-quoted literal string may contain newlines".to_owned())
        } else {
            Ok(contents.to_owned())
        }
    } else if let Some(suffix) = escaped.strip_prefix(r#"""""#) {
        // Multiline double-quoted string. Can contain escape sequences.
        let Some(contents) = suffix.strip_suffix(r#"""""#) else {
            return Err("Missing ending of multiline double-quoted string".to_owned());
        };
        unescape(contents)
    } else if let Some(suffix) = escaped.strip_prefix('"') {
        // Simple double-quoted string. Can contain escape sequences.
        let Some(contents) = suffix.strip_suffix('"') else {
            return Err("Double-quoted string must end with a double quote".to_owned());
        };
        if contents.contains('\n') {
            Err("Double-quoted string may contain newlines".to_owned())
        } else {
            unescape(contents)
        }
    } else {
        Err("String must start with a quote (single or double)".to_owned())
    }
}

fn unescape(s: &str) -> Result<String, String> {
    let mut chars = s.chars().peekable();

    let mut output = String::with_capacity(s.len());

    while let Some(chr) = chars.next() {
        if chr == '\\' {
            let Some(chr) = chars.next() else {
                return Err("String ended with a backslash".to_owned());
            };

            output.push(match chr {
                ' ' => ' ',
                '"' => '"',
                '/' => '/',
                '\'' => '\'',
                '\\' => '\\',
                '`' => '`',
                '$' => '$',
                'a' => '\u{07}',
                'b' => '\u{08}',
                'e' | 'E' => '\u{1B}',
                'f' => '\u{0C}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\u{0B}',
                '\n' => {
                    // Escaped newline => ignore newline, and the following whitespace:
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }
                    continue;
                }
                'u' => parse_braces_with_unicode(&mut chars)?,

                _ => return Err(format!("Unknown escape sequence: \\{chr}")),
            });
        } else {
            output.push(chr);
        }
    }

    Ok(output)
}

/// Parses e.g. `{1F600}` into a Unicode character.
fn parse_braces_with_unicode<I>(chars: &mut I) -> Result<char, String>
where
    I: Iterator<Item = char>,
{
    if chars.next() != Some('{') {
        return Err("\\u must be followed by an opening brace, e.g. \\u{1F600}".to_owned());
    }

    let mut num_chars = 0;
    let mut number: u32 = 0;

    for c in chars {
        if c == '}' {
            return char::from_u32(number)
                .ok_or_else(|| format!("Invalid Unicode code point: {number:0X}"));
        }

        if let Some(digit) = c.to_digit(16) {
            if num_chars >= 8 {
                return Err(format!(
                    "Unicode escape sequence is too long: {number:0X} (max 8 hex digits allowed)"
                ));
            }
            number = number * 16 + digit;
            num_chars += 1;
        } else if c == '_' {
            // Ignore underscores in the hex sequence.
        } else {
            return Err(format!("Invalid character in Unicode escape sequence: {c}"));
        }
    }

    Err("Unicode escape sequence must end with a closing brace '}'".to_owned())
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

#[test]
fn test_unescape() {
    assert_eq!(
        unescape_and_unquote(r#""normal \n string""#).unwrap(),
        "normal \n string",
    );
    assert_eq!(
        unescape_and_unquote(r#"'raw "string" \ yeah'"#).unwrap(),
        "raw \"string\" \\ yeah",
    );
    assert_eq!(
        unescape_and_unquote("'''Verbatim\nMultiline \"string\" \\ yeah'''").unwrap(),
        "Verbatim\nMultiline \"string\" \\ yeah",
    );
    assert_eq!(
        unescape_and_unquote("'''\nVerbatim\nMultiline \"string\"'''").unwrap(),
        "Verbatim\nMultiline \"string\"",
        "Should strip leading newline"
    );
    assert_eq!(
        unescape_and_unquote("\"\"\"Multi\\\n  line\n  String\n\"\"\"").unwrap(),
        "Multiline\n  String\n",
    );
}
