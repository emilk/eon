use std::hash::Hash;

use crate::Result;

/// Represents a number (float, integer, …)
#[derive(Debug, Clone)]
pub struct Number(NumberImpl);

#[derive(Debug, Clone)]
enum NumberImpl {
    I128(i128),
    U128(u128),

    // Having this separately for `f64` allows us to encode `f32` using fewer decimals.
    F32(f32),

    F64(f64),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(a), Some(b)) = (self.as_i128(), other.as_i128()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_u128(), other.as_u128()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) {
            a.is_nan() && b.is_nan() || a == b
        } else {
            false // different types
        }
    }
}

impl Eq for Number {}

impl Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(n) = self.as_i128() {
            n.hash(state);
        } else if let Some(n) = self.as_u128() {
            n.hash(state);
        } else if let Some(n) = self.as_f64() {
            if n == 0.0 {
                0_u64.hash(state);
            } else if n.is_nan() {
                1_u64.hash(state);
            } else if n == f64::NEG_INFINITY {
                2_u64.hash(state);
            } else if n == f64::INFINITY {
                3_u64.hash(state);
            } else {
                n.to_bits().hash(state);
            }
        } else {
            // We should never get here
        }
    }
}

impl std::str::FromStr for Number {
    type Err = String;

    fn from_str(mut string: &str) -> Result<Self, Self::Err> {
        if string.contains('_') {
            // We allow _ as thousands separators:
            return Self::from_str(string.replace('_', "").as_str());
        }

        match string {
            "+nan" => {
                return Ok(Self(NumberImpl::F32(f32::NAN)));
            }
            "-inf" => {
                return Ok(Self(NumberImpl::F32(f32::NEG_INFINITY)));
            }
            "+inf" => {
                return Ok(Self(NumberImpl::F32(f32::INFINITY)));
            }
            _ => {}
        }

        let sign = if let Some(rest) = string.strip_prefix('+') {
            string = rest;
            1
        } else if let Some(rest) = string.strip_prefix('-') {
            string = rest;
            -1
        } else {
            1
        };

        if string.to_lowercase() == "nan" {
            return Err("NaN must be written as '+nan'".to_owned());
        }

        let unsigned = if let Some(binary) = string.strip_prefix("0b") {
            let number = u128::from_str_radix(binary, 2)
                .map_err(|_err| "Failed to parse binary number. Expected '0b…'".to_owned())?;
            NumberImpl::U128(number)
        } else if let Some(hex) = string.strip_prefix("0x") {
            let number = u128::from_str_radix(hex, 16)
                .map_err(|_err| "Failed to parse hexadecimal number. Expected '0x…'".to_owned())?;
            NumberImpl::U128(number)
        } else if string.contains('.') || string.contains('e') {
            let as_f64 = string.parse::<f64>().map_err(|_err| {
                "Failed to parse float number. Expected a valid float.".to_owned()
            })?;
            let as_f32 = as_f64 as f32;
            if as_f32 as f64 == as_f64 {
                NumberImpl::F32(as_f32)
            } else {
                NumberImpl::F64(as_f64)
            }
        } else {
            NumberImpl::U128(
                string
                    .parse()
                    .map_err(|_err| "Not a valid number".to_owned())?,
            )
        };

        if sign == -1 {
            Self(unsigned)
                .try_negate()
                .ok_or_else(|| "Number too small".to_owned())
        } else {
            Ok(Self(unsigned))
        }
    }
}

impl Number {
    /// Returns None if the negation cannot be represented
    pub fn try_negate(&self) -> Option<Self> {
        match self.0 {
            NumberImpl::I128(value) => {
                if value == i128::MIN {
                    None // negation would overflow
                } else {
                    Some(NumberImpl::I128(-value))
                }
            }
            NumberImpl::U128(value) => {
                if value <= i128::MAX as u128 {
                    Some(NumberImpl::I128(-(value as i128)))
                } else {
                    None // negation would overflow
                }
            }
            NumberImpl::F32(value) => Some(NumberImpl::F32(-value)),
            NumberImpl::F64(value) => Some(NumberImpl::F64(-value)),
        }
        .map(Self)
    }

    /// Returns the value iff it can be represented without narrowing.
    pub fn as_i64(&self) -> Option<i64> {
        match self.0 {
            NumberImpl::I128(n) => i64::try_from(n).ok(),
            NumberImpl::U128(n) => i64::try_from(n).ok(),
            NumberImpl::F32(n) => {
                let i = n.round() as i64;
                if n == i as f32 { Some(i) } else { None }
            }
            NumberImpl::F64(n) => {
                let i = n.round() as i64;
                if n == i as f64 { Some(i) } else { None }
            }
        }
    }

    /// Returns the value iff it can be represented without narrowing.
    pub fn as_u64(&self) -> Option<u64> {
        match self.0 {
            NumberImpl::I128(n) => u64::try_from(n).ok(),
            NumberImpl::U128(n) => u64::try_from(n).ok(),
            NumberImpl::F32(n) => {
                let i = n.round() as u64;
                if n == i as f32 { Some(i) } else { None }
            }
            NumberImpl::F64(n) => {
                let i = n.round() as u64;
                if n == i as f64 { Some(i) } else { None }
            }
        }
    }

    /// Returns the value iff it can be represented without narrowing.
    pub fn as_i128(&self) -> Option<i128> {
        match self.0 {
            NumberImpl::I128(n) => Some(n),
            NumberImpl::U128(n) => i128::try_from(n).ok(),
            NumberImpl::F32(n) => {
                let i = n.round() as i128;
                if n == i as f32 { Some(i) } else { None }
            }
            NumberImpl::F64(n) => {
                let i = n.round() as i128;
                if n == i as f64 { Some(i) } else { None }
            }
        }
    }

    /// Returns the value iff it can be represented without narrowing.
    pub fn as_u128(&self) -> Option<u128> {
        match self.0 {
            NumberImpl::I128(n) => u128::try_from(n).ok(),
            NumberImpl::U128(n) => Some(n),
            NumberImpl::F32(n) => {
                let i = n.round() as u128;
                if n == i as f32 { Some(i) } else { None }
            }
            NumberImpl::F64(n) => {
                let i = n.round() as u128;
                if n == i as f64 { Some(i) } else { None }
            }
        }
    }

    /// Returns the value iff it can be represented without narrowing.
    pub fn as_f64(&self) -> Option<f64> {
        match self.0 {
            NumberImpl::I128(n) => {
                if n as f32 as i128 == n {
                    Some(n as f64)
                } else {
                    None
                }
            }
            NumberImpl::U128(n) => {
                if n as f32 as u128 == n {
                    Some(n as f64)
                } else {
                    None
                }
            }
            NumberImpl::F32(n) => Some(n as f64),
            NumberImpl::F64(n) => Some(n),
        }
    }
}

impl From<i8> for Number {
    #[inline]
    fn from(value: i8) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i16> for Number {
    #[inline]
    fn from(value: i16) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i32> for Number {
    #[inline]
    fn from(value: i32) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i64> for Number {
    #[inline]
    fn from(value: i64) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i128> for Number {
    #[inline]
    fn from(value: i128) -> Self {
        Self(NumberImpl::I128(value))
    }
}

impl From<u8> for Number {
    #[inline]
    fn from(value: u8) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u16> for Number {
    #[inline]
    fn from(value: u16) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u32> for Number {
    #[inline]
    fn from(value: u32) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u64> for Number {
    #[inline]
    fn from(value: u64) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u128> for Number {
    #[inline]
    fn from(value: u128) -> Self {
        Self(NumberImpl::U128(value))
    }
}

impl From<f32> for Number {
    #[inline]
    fn from(value: f32) -> Self {
        Self(NumberImpl::F32(value))
    }
}

impl From<f64> for Number {
    #[inline]
    fn from(value: f64) -> Self {
        Self(NumberImpl::F64(value))
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            NumberImpl::I128(n) => n.fmt(f),

            NumberImpl::U128(n) => n.fmt(f),

            NumberImpl::F32(n) => {
                if n.is_nan() {
                    "+nan".fmt(f)
                } else if *n == f32::NEG_INFINITY {
                    "-inf".fmt(f)
                } else if *n == f32::INFINITY {
                    "+inf".fmt(f)
                } else {
                    n.fmt(f) // TODO: always include a decimal point?
                }
            }

            NumberImpl::F64(n) => {
                if n.is_nan() {
                    "+nan".fmt(f)
                } else if *n == f64::NEG_INFINITY {
                    "-inf".fmt(f)
                } else if *n == f64::INFINITY {
                    "+inf".fmt(f)
                } else {
                    n.fmt(f) // TODO: always include a decimal point?
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use crate::Value;

    #[test]
    fn test_parse_number() {
        let integers = [
            ("0", 0),
            ("-0", 0),
            ("-1", -1),
            ("+1", 1),
            ("123_456_789", 123_456_789),
            ("0xe", 0xe),
            ("0x1f", 0x1f),
            ("0b101010", 0b101010),
            ("-0b101010", -0b101010),
            ("+0b101010", 0b101010),
        ];

        for (string, expected) in integers {
            eprintln!("Parsing integer: {string:?}");
            let value = Value::from_str(string).expect("Failed to parse number");
            let number = value.as_number().expect("Expected a number");
            assert_eq!(number.as_i128(), Some(expected));
        }
    }
}
