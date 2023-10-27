use std::fmt::Display;

use crate::config::InfluxConfig;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, SharedString, Unit};
use reqwest::{Client, RequestBuilder};

mod config;
mod metric;
mod types;

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
                    config
                        .consistency
                        .as_ref()
                        .map(|c| ("consistency", c.to_string())),
                    config
                        .precision
                        .as_ref()
                        .map(|p| ("precision", p.to_string())),
                    config
                        .retention_policy
                        .as_ref()
                        .map(|rp| ("rp", rp.clone())),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<(&str, String)>>();

                client
                    .post(format!("{}/write", config.endpoint))
                    .query(&parameters)
            }
            InfluxConfig::V2(config) => {
                todo!()
            }
        };

        InfluxClient { client, request }
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
