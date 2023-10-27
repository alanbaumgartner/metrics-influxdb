use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, SharedString, Unit};
use metrics_util::parse_quantiles;
use metrics_util::registry::{Registry};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::config::InfluxConfig;
use crate::distribution::DistributionBuilder;
use crate::error::{InfluxError, Result};
use crate::metric::Metric;
use crate::registry::AtomicStorage;

pub struct InfluxClient {
    recorder: Arc<InfluxRecorder>,
}

impl InfluxClient {
    pub fn new(config: impl InfluxConfig) -> Self {
        let client = Client::new();
        let request = config.request(&client);
        InfluxClient {
            recorder: Arc::new(InfluxRecorder::new(client, request)),
        }
    }

    pub fn start(&self, delay: Duration) {
        let recorder = self.recorder.clone();
        tokio::spawn(async move {
            loop {
                sleep(delay).await;

                let counter_gauges = recorder
                    .registry
                    .get_counter_handles()
                    .iter()
                    .chain(recorder.registry.get_gauge_handles().iter())
                    .map(|metric| metric.into())
                    .collect::<Vec<Metric>>();

                let histograms = recorder
                    .registry
                    .get_histogram_handles()
                    .iter()
                    .map(|(key, value)| {
                        let mut distribution = recorder.distribution_builder.get_distribution();
                        value.clear_with(|samples| distribution.record_samples(samples));
                        (key, distribution)
                    })
                    .map(|metric| metric.into())
                    .collect::<Vec<Metric>>();

                let metrics = counter_gauges.into_iter().chain(histograms).join("\n");

                match recorder.write_metrics(metrics).await {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        });
    }

    pub async fn write(&self, metric: &Metric) -> Result<()> {
        self.recorder.write_metrics(metric.to_string()).await
    }
}

pub struct InfluxRecorder {
    client: Client,
    request: RequestBuilder,
    registry: Registry<Key, AtomicStorage>,
    distribution_builder: DistributionBuilder,
}

impl InfluxRecorder {
    pub fn new(client: Client, request: RequestBuilder) -> InfluxRecorder {
        let quantiles = parse_quantiles(&[0.0, 0.5, 0.9, 0.95, 0.99, 0.999, 1.0]);
        InfluxRecorder {
            client,
            request,
            registry: Registry::new(AtomicStorage),
            distribution_builder: DistributionBuilder::new(quantiles, None),
        }
    }

    async fn write_metrics(&self, metrics: String) -> Result<()> {
        let response = self
            .request
            .try_clone()
            .unwrap()
            .body(metrics)
            .send()
            .await?;

        let status = response.status();
        let influx_api_response = response.json::<InfluxApiResponse>().await?;

        match status {
            status if status.is_success() => Ok(()),
            err if err.as_u16() == 401 => Err(InfluxError::AuthenticationError {
                error: influx_api_response.message.unwrap(),
            }),
            err if err.as_u16() == 403 => Err(InfluxError::AuthorizationError {
                error: influx_api_response.message.unwrap(),
            }),
            err if err.as_u16() == 413 => Err(InfluxError::ContentTooLarge {
                error: influx_api_response.message.unwrap(),
            }),
            _status => {
                todo!()
            }
        }
    }
}

impl Recorder for InfluxRecorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        unimplemented!("InfluxDB ILP does not support descriptions.")
    }

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        unimplemented!("InfluxDB ILP does not support descriptions.")
    }

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        unimplemented!("InfluxDB ILP does not support descriptions.")
    }

    fn register_counter(&self, key: &Key) -> Counter {
        self.registry
            .get_or_create_counter(key, |counter| counter.to_owned().into())
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        self.registry
            .get_or_create_gauge(key, |gauge| gauge.to_owned().into())
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        self.registry
            .get_or_create_histogram(key, |histogram| histogram.to_owned().into())
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct InfluxApiResponse {
    message: Option<String>,
    op: Option<String>,
    err: Option<String>,
    line: Option<i32>,
    max_len: Option<i32>,
}
