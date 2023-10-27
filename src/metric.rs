use std::fmt::{Display, Formatter};

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::Type;

const RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([,="])"#).unwrap());

#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    measurement: String,
    fields: Vec<(String, Type)>,
    tags: Vec<(String, Type)>,
    timestamp: Option<u128>,
}

impl Metric {
    pub fn new(measurement: impl Into<String>) -> Self {
        Metric {
            measurement: measurement.into(),
            fields: vec![],
            tags: vec![],
            timestamp: None,
        }
    }

    pub fn field(mut self, field: impl Into<String>, value: impl Into<Type>) -> Self {
        self.fields.push((field.into(), value.into()));
        self
    }

    pub fn tag(mut self, tag: impl Into<String>, value: impl Into<Type>) -> Self {
        self.tags.push((tag.into(), value.into()));
        self
    }
}

impl Display for Metric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let measurement = if self.tags.is_empty() {
            escape_string(self.measurement.clone())
        } else {
            escape_string(self.measurement.clone()) + ","
        };

        let tags = self
            .tags
            .iter()
            .map(|(tag, value)| {
                format!(
                    "{}={}",
                    escape_string(tag),
                    escape_string(value.to_string())
                )
            })
            .join(",");

        let fields = self
            .fields
            .iter()
            .map(|(field, value)| {
                format!(
                    "{}={}",
                    escape_string(field),
                    escape_string(value.to_string())
                )
            })
            .join(",");

        let ilp = vec![tags, fields].join(" ");
        write!(f, "{measurement}{ilp}")
    }
}

fn escape_string(value: impl Into<String>) -> String {
    let value = value.into();
    let value = RE.replace_all(value.as_str(), "\\${1}").to_string();
    if value.contains(" ") {
        format!("\"{value}\"")
    } else {
        value.clone()
    }
}

mod test {
    use crate::metric::{escape_string, Metric};

    #[test]
    fn test_escape_string() {
        assert_eq!(
            "\"tag\\=Hello\\, world!\"",
            escape_string("tag=Hello, world!")
        )
    }

    #[test]
    fn display() {
        let expected = "weather,location=us-midwest temperature=\"too warm\"".to_string();

        let metric = Metric::new("weather")
            .tag("location", "us-midwest")
            .field("temperature", "too warm");

        assert_eq!(expected, metric.to_string());
    }
}
