use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Type {
    Boolean(bool),
    Float(f64),
    SignedInteger(i64),
    UnsignedInteger(u64),
    Text(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Type::Boolean(value) => value.to_string(),
            Type::Float(value) => value.to_string(),
            Type::SignedInteger(value) => format!("{}i", value.to_string()),
            Type::UnsignedInteger(value) => format!("{}i", value.to_string()),
            Type::Text(value) => value.clone(),
        };
        write!(f, "{str}")
    }
}

impl From<bool> for Type {
    fn from(value: bool) -> Self {
        Type::Boolean(value)
    }
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        Type::Text(value)
    }
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        Type::Text(value.to_string())
    }
}

impl From<f32> for Type {
    fn from(value: f32) -> Self {
        Type::Float(value as f64)
    }
}

impl From<f64> for Type {
    fn from(value: f64) -> Self {
        Type::Float(value)
    }
}

impl From<u8> for Type {
    fn from(value: u8) -> Self {
        Type::UnsignedInteger(value as u64)
    }
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        Type::UnsignedInteger(value as u64)
    }
}

impl From<u32> for Type {
    fn from(value: u32) -> Self {
        Type::UnsignedInteger(value as u64)
    }
}

impl From<u64> for Type {
    fn from(value: u64) -> Self {
        Type::UnsignedInteger(value)
    }
}

impl From<i8> for Type {
    fn from(value: i8) -> Self {
        Type::SignedInteger(value as i64)
    }
}

impl From<i16> for Type {
    fn from(value: i16) -> Self {
        Type::SignedInteger(value as i64)
    }
}

impl From<i32> for Type {
    fn from(value: i32) -> Self {
        Type::SignedInteger(value as i64)
    }
}

impl From<i64> for Type {
    fn from(value: i64) -> Self {
        Type::SignedInteger(value)
    }
}

impl From<usize> for Type {
    fn from(value: usize) -> Self {
        Type::UnsignedInteger(value as u64)
    }
}
