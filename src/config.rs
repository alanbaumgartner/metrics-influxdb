use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Consistency {
    All,
    Any,
    One,
    Quorum,
}

impl Display for Consistency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Consistency::All => "all",
            Consistency::Any => "any",
            Consistency::One => "one",
            Consistency::Quorum => "quorum",
        };
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone)]
pub enum Precision {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

impl Display for Precision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Precision::Nanoseconds => "ns",
            Precision::Microseconds => "u",
            Precision::Milliseconds => "ms",
            Precision::Seconds => "s",
            Precision::Minutes => "m",
            Precision::Hours => "h",
        };
        write!(f, "{str}")
    }
}

pub enum InfluxConfig {
    V1(InfluxV1Config),
    V2(InfluxV2Config),
}

pub(crate) struct InfluxV1Config {
    pub(crate) endpoint: String,
    pub(crate) db: String,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) retention_policy: Option<String>,
    pub(crate) precision: Option<Precision>,
    pub(crate) consistency: Option<Consistency>,
}

pub(crate) struct InfluxV2Config {
    pub(crate) endpoint: String,
    pub(crate) bucket: String,
    pub(crate) org: String,
    pub(crate) precision: Precision,
}