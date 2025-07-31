use crate::Result;

fn is_keyword(string: &str) -> bool {
    matches!(string, "true" | "false" | "null")
}

/// Returns `true` if the string does matches `[a-zA-Z_][a-zA-Z0-9_]*`
pub fn is_valid_identifier(string: &str) -> bool {
    if is_keyword(string) {
        return false; // We need to quote these to avoid confusion with the special keyword values
    }

    let mut chars = string.chars();

    // Check first character:
    if chars
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '_')
    {
        return false;
    }

    // Check the rest:
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

/// Format a string into Eon, adding quotes and escaping as needed.
///
/// The exact type of quoting (single, double, multiline basic, or multiline literal) will be determined automatically based on the content of the string.
pub fn escape_and_quote(raw: &str) -> String {
    // TODO: smartly choose between all four types of strings: double, single, multiline basic, and multiline literal.

    let must_be_double_quoted = raw
        .chars()
        .any(|c| c.is_control() || matches!(c, '\'' | '\n' | '\r' | '\t'));

    if must_be_double_quoted {
        return doubel_quote(raw);
    }

    let would_be_shorter_if_literal = raw
        .chars()
        .any(|c| c.is_control() || matches!(c, '"' | '\\'));

    if would_be_shorter_if_literal {
        // This would benefit from using single-quoted literal strings:
        format!("'{raw}'")
    } else {
        // The default
        doubel_quote(raw)
    }
}

fn doubel_quote(raw: &str) -> String {
    format!("{raw:?}")
}

/// Remove the quotes and unescape the string.
pub fn unescape_and_unquote(escaped: &str) -> Result<String, String> {
    if escaped.contains('\r') {
        // Handle Windows newlines by stripping all `\r` characters,
        // turning `\r\n` into `\n`.
        return unescape_and_unquote(&escaped.replace('\r', ""));
    }

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
fn test_is_valid_identifier() {
    assert!(!is_valid_identifier(""));
    assert!(is_valid_identifier("X"));
    assert!(is_valid_identifier("valid_identifier"));
    assert!(is_valid_identifier("valid123"));
    assert!(is_valid_identifier("_valid"));
    assert!(!is_valid_identifier("123invalid"));
    assert!(!is_valid_identifier("invalid-identifier"));
    assert!(!is_valid_identifier("true"));
    assert!(!is_valid_identifier("false"));
    assert!(!is_valid_identifier("null"));
    assert!(!is_valid_identifier("invalid identifier"));
    assert!(!is_valid_identifier("invalid@identifier"));
    assert!(!is_valid_identifier("invalid!identifier"));
    assert!(!is_valid_identifier("invalid#identifier"));
}

#[test]
fn test_escape() {
    assert_eq!(escape_and_quote("normal"), r#""normal""#);
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes'"#),
        r#""a string with 'single-quotes'""#
    );
    assert_eq!(
        escape_and_quote(r#"a string with "double-quotes" and \ backslash"#),
        r#"'a string with "double-quotes" and \ backslash'"#
    );
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes' and "double-quotes""#),
        r#""a string with 'single-quotes' and \"double-quotes\"""#
    );
    assert_eq!(
        escape_and_quote("a string with tab \t"),
        r#""a string with tab \t""#
    );
    assert_eq!(
        escape_and_quote("a string with tab \t and \"double-quotes\""),
        r#""a string with tab \t and \"double-quotes\"""#
    );
    assert_eq!(
        escape_and_quote("a string with tab \t and \"double-quotes\""),
        r#""a string with tab \t and \"double-quotes\"""#
    );

    assert_eq!(
        escape_and_quote(r#"C:\System32\foo.dll"#),
        r#"'C:\System32\foo.dll'"#
    );
    assert_eq!(
        escape_and_quote(r#"I use "quotes" in this string"#),
        r#"'I use "quotes" in this string'"#
    );
    assert_eq!(
        escape_and_quote(r#"[+\-0-9\.][0-9a-zA-Z\.+\-_]*"#),
        r#"'[+\-0-9\.][0-9a-zA-Z\.+\-_]*'"#
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
