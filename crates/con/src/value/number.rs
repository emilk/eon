use crate::Result;

/// Represents a number (float, integer, …)
#[derive(Debug, Clone, PartialEq)]
pub struct Number(NumberImpl);

#[derive(Debug, Clone, PartialEq)]
enum NumberImpl {
    I128(i128),
    U128(u128),

    // Having this seperatedly allows us to encode f32 using less precision.
    F32(f32),

    F64(f64),

    /// Yet-to-be parsed.
    String(String),
}

impl Number {
    pub(crate) fn try_parse(source: &str, string: &str) -> Result<Self> {
        Ok(Self(NumberImpl::String(string.to_owned())))
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
            NumberImpl::String(_) => None, // TODO: parse string
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
            NumberImpl::String(_) => None, // TODO: parse string
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
            NumberImpl::String(_) => None, // TODO: parse string
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
            NumberImpl::String(_) => None, // TODO: parse string
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
            NumberImpl::String(_) => None, // TODO: parse string
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
                    write!(f, "NaN")
                } else if *n == f32::NEG_INFINITY {
                    write!(f, "-inf")
                } else if *n == f32::INFINITY {
                    write!(f, "+inf")
                } else {
                    // TODO: always include a decimal point?
                    n.fmt(f)
                }
            }

            NumberImpl::F64(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if *n == f64::NEG_INFINITY {
                    write!(f, "-inf")
                } else if *n == f64::INFINITY {
                    write!(f, "+inf")
                } else {
                    // TODO: always include a decimal point?
                    n.fmt(f)
                }
            }

            NumberImpl::String(s) => s.fmt(f),
        }
    }
}
