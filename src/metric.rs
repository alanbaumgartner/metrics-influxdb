use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use itertools::Itertools;
use metrics::{Key, Label};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::io::AsyncReadExt;

use crate::distribution::Distribution;
use crate::types::Type;

const RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([,="])"#).unwrap());

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl From<(&Key, &Arc<AtomicU64>)> for Metric {
    fn from(value: (&Key, &Arc<AtomicU64>)) -> Self {
        let (key, value) = value;
        let tags = parse_labels(key.labels());
        Metric {
            measurement: key.name().to_string(),
            fields: vec![("value".to_owned(), value.load(Ordering::Relaxed).into())],
            tags,
            timestamp: None,
        }
    }
}

impl From<(&Key, Distribution)> for Metric {
    fn from((key, value): (&Key, Distribution)) -> Self {
        let tags = parse_labels(key.labels());

        let fields = match value {
            Distribution::Histogram(histogram) => vec![
                ("sum".to_owned(), histogram.sum().into()),
                ("count".to_owned(), histogram.count().into()),
            ]
            .into_iter()
            .chain(
                histogram
                    .buckets()
                    .into_iter()
                    .map(|(label, count)| (format!("{:.2}", label), count.into())),
            )
            .collect(),
            Distribution::Summary(summary, quantiles, sum) => vec![
                ("sum".to_owned(), sum.into()),
                ("count".to_owned(), summary.count().into()),
            ]
            .into_iter()
            .chain(
                quantiles
                    .iter()
                    .map(|quantile| (quantile.label().to_owned(), quantile.value().into())),
            )
            .collect(),
        };

        Metric {
            measurement: key.name().to_string(),
            fields,
            tags,
            timestamp: None,
        }
    }
}

fn parse_labels(labels: Iter<Label>) -> Vec<(String, Type)> {
    labels
        .map(|label| {
            let (key, value) = label.clone().into_parts();
            (key.to_string(), value.to_string().into())
        })
        .collect()
}

mod test {
    use crate::metric::{escape_string, Metric};

    #[test]
    fn test_escape_string() {
        assert_eq!(
            "\"test\\=Hello\\, world!\"",
            escape_string("test=Hello, world!")
        )
    }

    #[test]
    fn test_display() {
        let expected = r#"test,string=test escaped_string="te\ st",float_value=10.1,unsigned_value=10i,signed_value=10i"#.to_owned();

        let metric = Metric::new("test")
            .tag("string", "test")
            .field("escaped_string", "te\\ st")
            .field("float_value", 10.1)
            .field("unsigned_value", 10u8)
            .field("signed_value", 10i8);

        assert_eq!(expected, metric.to_string());
    }
}
