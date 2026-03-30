#![no_main]

use std::str::FromStr as _;

use arbitrary::{Arbitrary, Unstructured};
use eon::Value;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Clone, Copy, Debug)]
enum HiddenChar {
    SoftHyphen,
    CombiningGraphemeJoiner,
    ArabicLetterMark,
    ZeroWidthSpace,
    ZeroWidthJoiner,
    LeftToRightMark,
    RightToLeftMark,
    RightToLeftOverride,
    WordJoiner,
    LeftToRightIsolate,
    PopDirectionalIsolate,
    VariationSelector16,
    ByteOrderMark,
}

impl HiddenChar {
    fn as_char(self) -> char {
        match self {
            Self::SoftHyphen => '\u{00AD}',
            Self::CombiningGraphemeJoiner => '\u{034F}',
            Self::ArabicLetterMark => '\u{061C}',
            Self::ZeroWidthSpace => '\u{200B}',
            Self::ZeroWidthJoiner => '\u{200D}',
            Self::LeftToRightMark => '\u{200E}',
            Self::RightToLeftMark => '\u{200F}',
            Self::RightToLeftOverride => '\u{202E}',
            Self::WordJoiner => '\u{2060}',
            Self::LeftToRightIsolate => '\u{2066}',
            Self::PopDirectionalIsolate => '\u{2069}',
            Self::VariationSelector16 => '\u{FE0F}',
            Self::ByteOrderMark => '\u{FEFF}',
        }
    }
}

#[derive(Arbitrary, Clone, Copy, Debug)]
enum Placement {
    Comment,
    BasicString,
    LiteralString,
    VariantName,
}

#[derive(Arbitrary, Debug)]
struct Case {
    prefix: String,
    suffix: String,
    hidden: HiddenChar,
    placement: Placement,
}

fuzz_target!(|data: &[u8]| {
    let Ok(case) = Case::arbitrary(&mut Unstructured::new(data)) else {
        return;
    };

    let inserted = format!("{}{}{}", case.prefix, case.hidden.as_char(), case.suffix);
    let quoted = sanitize_for_quoted_context(&inserted);
    let source = match case.placement {
        Placement::Comment => format!("// {}\nvalue: 1", inserted),
        Placement::BasicString => format!("value: \"{}\"", quoted),
        Placement::LiteralString => {
            let sanitized = inserted.replace('\'', "_").replace('\n', "_").replace('\r', "_");
            format!("value: '{}'", sanitized)
        }
        Placement::VariantName => format!("mode: \"{}\"()", quoted),
    };

    assert!(
        Value::from_str(&source).is_err(),
        "legacy parser accepted hidden Unicode in {source:?}"
    );
    assert!(
        Value::from_str_with_core(&source).is_err(),
        "core parser accepted hidden Unicode in {source:?}"
    );
    assert!(
        eon::reformat(&source, &Default::default()).is_err(),
        "formatter accepted hidden Unicode in {source:?}"
    );
});

fn sanitize_for_quoted_context(input: &str) -> String {
    input
        .replace('\\', "_")
        .replace('"', "_")
        .replace('\n', "_")
        .replace('\r', "_")
}
