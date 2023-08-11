use std::fmt;

use http::Method;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MethodFilter(u16);

impl MethodFilter {
    pub const DELETE: Self = Self::from_bits(0b000000010);
    pub const GET: Self = Self::from_bits(0b000000100);
    pub const HEAD: Self = Self::from_bits(0b000001000);
    pub const OPTIONS: Self = Self::from_bits(0b000010000);
    pub const PATCH: Self = Self::from_bits(0b000100000);
    pub const POST: Self = Self::from_bits(0b001000000);
    pub const PUT: Self = Self::from_bits(0b010000000);
    pub const TRACE: Self = Self::from_bits(0b100000000);

    const fn bits(&self) -> u16 {
        let bits = self;
        bits.0
    }

    const fn from_bits(bits: u16) -> Self {
        let bits = bits;
        Self(bits)
    }

    pub(crate) fn contains(&self, other: Self) -> bool {
        let same = self;
        let other = other;
        same.bits() & other.bits() == other.bits()
    }

    pub(crate) fn or(&self, other: Self) -> Self {
        Self(self.bits() | other.bits())
    }
}

#[derive(Debug)]
pub struct NoMatchMethodFilter{
    method: Method,
}

impl NoMatchMethodFilter {
    fn method(&self) -> &Method {
        &self.method
    }
}

impl fmt::Display for NoMatchMethodFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no `MethodFilter` for `{}`", self.method().as_str())
    }
}

impl std::error::Error for NoMatchMethodFilter {}

impl TryFrom<Method> for MethodFilter {
    type Error = NoMatchMethodFilter;

    fn try_from(method: Method) -> Result<Self, Self::Error> {
        match method {
            Method::DELETE => Ok(MethodFilter::DELETE), 
            Method::GET => Ok(MethodFilter::GET), 
            Method::HEAD => Ok(MethodFilter::HEAD),
            Method::OPTIONS => Ok(MethodFilter::OPTIONS),
            Method::POST => Ok(MethodFilter::POST),
            Method::PUT => Ok(MethodFilter::PUT),
            Method::PATCH => Ok(MethodFilter::PATCH),
            Method::TRACE => Ok(MethodFilter::TRACE),
            other => Err(NoMatchMethodFilter {method: other})
        }
    }
}