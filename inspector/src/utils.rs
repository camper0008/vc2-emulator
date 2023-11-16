use std::{num::ParseIntError, str::FromStr};

pub trait FromStrRadix
where
    Self: Sized,
{
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, std::num::ParseIntError>;
}

impl FromStrRadix for u8 {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(value, radix)
    }
}

impl FromStrRadix for u32 {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(value, radix)
    }
}

impl FromStrRadix for usize {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(value, radix)
    }
}

pub fn parse_integer<T: FromStrRadix + FromStr<Err = ParseIntError>>(
    value: &str,
) -> Result<T, ParseIntError> {
    if value.starts_with("0x") {
        T::from_str_radix(&value[2..], 16)
    } else if value.starts_with("0b") {
        T::from_str_radix(&value[2..], 2)
    } else {
        value.parse()
    }
}
