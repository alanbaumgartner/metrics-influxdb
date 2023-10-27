use std::fmt::Display;
use std::time::SystemTime;

use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, SharedString, Unit};
use reqwest::{Client, RequestBuilder};
use crate::config::InfluxConfig;

use crate::types::Type;

mod types;
mod config;

#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    measurement: String,
    fields: Vec<(String, Type)>,
    tags: Vec<(String, Type)>,
    timestamp: u128,
}

impl Metric {
    pub fn new(measurement: impl Into<String>) -> Self {
        Metric {
            measurement: measurement.into(),
            fields: vec![],
            tags: vec![],
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
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


pub struct InfluxClient {
    client: Client,
    request: RequestBuilder,
}

impl InfluxClient {
    pub fn new(config: InfluxConfig) -> Self {
        let client = Client::new();

        let request = match config {
            InfluxConfig::V1(config) => {
                let parameters = vec![
                    Some(("db", config.db.clone())),
                    config.username.as_ref().map(|u| ("u", u.clone())),
                    config.password.as_ref().map(|p| ("p", p.clone())),
                    config.consistency.as_ref().map(|c| ("consistency", c.to_string())),
                    config.precision.as_ref().map(|p| ("precision", p.to_string())),
                    config.retention_policy.as_ref().map(|rp| ("rp", rp.clone())),
                ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(&str, String)>>();

                client.post(format!("{}/write", config.endpoint))
                    .query(&parameters)
            }
            InfluxConfig::V2(config) => {
                todo!()
            }
        };

        InfluxClient {
            client,
            request,
        }
    }
}

impl Recorder for InfluxClient {
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        todo!()
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        todo!()
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        todo!()
    }

    fn register_counter(&self, key: &Key) -> Counter {
        todo!()
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        todo!()
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        todo!()
    }
}

mod test {
    use crate::Metric;

    #[test]
    fn test() {
        let metric = Metric {
            measurement: "test".to_string(),
            fields: vec![("".to_string(), "".into())],
            tags: vec![],
            timestamp: 0,
        };
        let result = Metric::new("test").field("", "");
        assert_eq!(metric, result);
    }
}