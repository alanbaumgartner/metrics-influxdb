/// Source https://github.com/metrics-rs/metrics/blob/main/metrics-exporter-prometheus/src/distribution.rs
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use metrics_util::{Histogram, Quantile, Summary};
use quanta::Instant;

/// Distribution type.
#[derive(Clone)]
pub enum Distribution {
    /// A Prometheus histogram.
    ///
    /// Exposes "bucketed" values to Prometheus, counting the number of samples
    /// below a given threshold i.e. 100 requests faster than 20ms, 1000 requests
    /// faster than 50ms, etc.
    Histogram(Histogram),
    /// A Prometheus summary.
    ///
    /// Computes and exposes value quantiles directly to Prometheus i.e. 50% of
    /// requests were faster than 200ms, and 99% of requests were faster than
    /// 1000ms, etc.
    Summary(RollingSummary, Arc<Vec<Quantile>>, f64),
}

impl Distribution {
    /// Creates a histogram distribution.
    pub fn new_histogram(buckets: &[f64]) -> Distribution {
        let hist = Histogram::new(buckets).expect("buckets should never be empty");
        Distribution::Histogram(hist)
    }

    /// Creates a summary distribution.
    pub fn new_summary(quantiles: Arc<Vec<Quantile>>) -> Distribution {
        let summary = RollingSummary::default();
        Distribution::Summary(summary, quantiles, 0.0)
    }

    /// Records the given `samples` in the current distribution.
    pub fn record_samples(&mut self, samples: &[(f64, Instant)]) {
        match self {
            Distribution::Histogram(hist) => {
                hist.record_many(samples.iter().map(|(sample, _ts)| sample));
            }
            Distribution::Summary(hist, _, sum) => {
                for (sample, ts) in samples {
                    hist.add(*sample, *ts);
                    *sum += *sample;
                }
            }
        }
    }
}

/// Builds distributions for metric names based on a set of configured overrides.
#[derive(Debug)]
pub struct DistributionBuilder {
    quantiles: Arc<Vec<Quantile>>,
    buckets: Option<Vec<f64>>,
}

impl DistributionBuilder {
    /// Creates a new instance of `DistributionBuilder`.
    pub fn new(quantiles: Vec<Quantile>, buckets: Option<Vec<f64>>) -> DistributionBuilder {
        DistributionBuilder {
            quantiles: Arc::new(quantiles),
            buckets,
        }
    }

    /// Returns a distribution for the given metric key.
    pub fn get_distribution(&self) -> Distribution {
        if let Some(ref buckets) = self.buckets {
            return Distribution::new_histogram(buckets);
        }

        Distribution::new_summary(self.quantiles.clone())
    }

    /// Returns the distribution type for the given metric key.
    pub fn get_distribution_type(&self) -> &str {
        if self.buckets.is_some() {
            return "histogram";
        }
        "summary"
    }
}

#[derive(Clone)]
struct Bucket {
    begin: Instant,
    summary: Summary,
}

/// A `RollingSummary` manages a list of [Summary] so that old results can be expired.
#[derive(Clone)]
pub struct RollingSummary {
    // Buckets are ordered with the latest buckets first.  The buckets are kept in alignment based
    // on the instant of the first added bucket and the bucket_duration.  There may be gaps in the
    // bucket list.
    buckets: Vec<Bucket>,
    // Maximum number of buckets to track.
    max_buckets: usize,
    // Duration of values stored per bucket.
    bucket_duration: Duration,
    // This is the maximum duration a bucket will be kept.
    max_bucket_duration: Duration,
    // Total samples since creation of this summary.  This is separate from the Summary since it is
    // never reset.
    count: usize,
}

impl Default for RollingSummary {
    fn default() -> Self {
        RollingSummary::new(NonZeroU32::new(3).unwrap(), Duration::from_secs(20))
    }
}

impl RollingSummary {
    /// Create a new `RollingSummary` with the given number of `buckets` and `bucket-duration`.
    ///
    /// The summary will store quantiles over `buckets * bucket_duration` seconds.
    pub fn new(buckets: std::num::NonZeroU32, bucket_duration: Duration) -> RollingSummary {
        assert!(!bucket_duration.is_zero());
        let max_bucket_duration = bucket_duration * buckets.get();
        let max_buckets = buckets.get() as usize;

        RollingSummary {
            buckets: Vec::with_capacity(max_buckets),
            max_buckets,
            bucket_duration,
            max_bucket_duration,
            count: 0,
        }
    }

    /// Add a sample `value` to the `RollingSummary` at the time `now`.
    ///
    /// Any values that expire at the `value_ts` are removed from the `RollingSummary`.
    pub fn add(&mut self, value: f64, now: Instant) {
        // The count is incremented even if this value is too old to be saved in any bucket.
        self.count += 1;

        // If we can find a bucket that this value belongs in, then we can just add it in and be
        // done.
        for bucket in &mut self.buckets {
            let end = bucket.begin + self.bucket_duration;

            // If this value belongs in a future bucket...
            if now > bucket.begin + self.bucket_duration {
                break;
            }

            if now >= bucket.begin && now < end {
                bucket.summary.add(value);
                return;
            }
        }

        // Remove any expired buckets.
        if let Some(cutoff) = now.checked_sub(self.max_bucket_duration) {
            self.buckets.retain(|b| b.begin > cutoff);
        }

        if self.buckets.is_empty() {
            let mut summary = Summary::with_defaults();
            summary.add(value);
            self.buckets.push(Bucket {
                begin: now,
                summary,
            });
            return;
        }

        // Take the first bucket time as a reference.  Other buckets will be created at an offset
        // of this time.  We know this time is close to the value_ts, if it were much older the
        // bucket would have been removed.
        let reftime = self.buckets[0].begin;

        let mut summary = Summary::with_defaults();
        summary.add(value);

        // If the value is newer than the first bucket then count upwards to the new bucket time.
        let mut begin;
        if now > reftime {
            begin = reftime + self.bucket_duration;
            let mut end = begin + self.bucket_duration;
            while now < begin || now >= end {
                begin += self.bucket_duration;
                end += self.bucket_duration;
            }

            self.buckets.truncate(self.max_buckets - 1);
            self.buckets.insert(0, Bucket { begin, summary });
        } else {
            begin = reftime - self.bucket_duration;
            while now < begin {
                begin -= self.bucket_duration;
            }

            self.buckets.truncate(self.max_buckets - 1);
            self.buckets.push(Bucket { begin, summary });
            self.buckets.sort_unstable_by(|a, b| b.begin.cmp(&a.begin));
        }
    }

    /// Return a merged Summary of all items that are valid at `now`.
    ///
    /// # Warning
    ///
    /// The snapshot `Summary::count()` contains the total number of values considered in the
    /// Snapshot, which is not the full count of the `RollingSummary`.  Use `RollingSummary::count()`
    /// instead.
    pub fn snapshot(&self, now: Instant) -> Summary {
        let cutoff = now.checked_sub(self.max_bucket_duration);
        let mut acc = Summary::with_defaults();
        self.buckets
            .iter()
            .filter(|b| {
                if let Some(cutoff) = cutoff {
                    b.begin > cutoff
                } else {
                    true
                }
            })
            .map(|b| &b.summary)
            .fold(&mut acc, |acc, item| {
                acc.merge(item)
                    .expect("merge can only fail if summary config inconsistent");
                acc
            });
        acc
    }

    /// Whether or not this summary is empty.
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Gets the totoal number of samples this summary has seen so far.
    pub fn count(&self) -> usize {
        self.count
    }

    #[cfg(test)]
    fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets
    }
}
