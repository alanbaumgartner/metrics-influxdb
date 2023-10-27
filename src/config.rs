use derivative::Derivative;
use std::fmt::{Display, Formatter};

use derive_builder::Builder;
use reqwest::{Client, RequestBuilder};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum Consistency {
    #[default]
    One,
    Quorum,
    All,
    Any,
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

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum Precision {
    #[default]
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

pub trait InfluxConfig {
    fn parameters(&self) -> Vec<(&str, String)>;
    fn request(&self, client: &Client) -> RequestBuilder;
}

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Default, Clone, PartialEq, PartialOrd, Builder)]
#[builder(setter(into))]
#[builder(default)]
pub struct InfluxV1Config {
    pub(crate) endpoint: String,
    pub(crate) db: String,
    pub(crate) username: Option<String>,
    #[derivative(Debug = "ignore")]
    pub(crate) password: Option<String>,
    pub(crate) retention_policy: Option<String>,
    pub(crate) precision: Option<Precision>,
    pub(crate) consistency: Option<Consistency>,
}

impl InfluxConfig for InfluxV1Config {
    fn parameters(&self) -> Vec<(&str, String)> {
        vec![
            Some(("db", self.db.clone())),
            self.username.as_ref().map(|u| ("u", u.clone())),
            self.password.as_ref().map(|p| ("p", p.clone())),
            self.consistency
                .as_ref()
                .map(|c| ("consistency", c.to_string())),
            self.precision
                .as_ref()
                .map(|p| ("precision", p.to_string())),
            self.retention_policy.as_ref().map(|rp| ("rp", rp.clone())),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<(&str, String)>>()
    }

    fn request(&self, client: &Client) -> RequestBuilder {
        client
            .post(format!("{}/write", self.endpoint))
            .query(&self.parameters())
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Default, Clone, PartialEq, PartialOrd, Builder)]
#[builder(setter(into))]
#[builder(default)]
pub struct InfluxV2Config {
    pub(crate) endpoint: String,
    pub(crate) bucket: String,
    pub(crate) org: String,
    pub(crate) precision: Precision,
    pub(crate) username: Option<String>,
    #[derivative(Debug = "ignore")]
    pub(crate) password: Option<String>,
}

impl InfluxConfig for InfluxV2Config {
    fn parameters(&self) -> Vec<(&str, String)> {
        todo!()
    }

    fn request(&self, client: &Client) -> RequestBuilder {
        todo!()
    }
}

mod test {
    use crate::config::{
        Consistency, InfluxConfig, InfluxV1Config, InfluxV1ConfigBuilder, Precision,
    };

    #[test]
    fn test_v1_config() {
        let expected_config = InfluxV1Config {
            endpoint: "http://localhost:8086".to_owned(),
            db: "metrics".to_owned(),
            username: Some("username".to_owned()),
            password: Some("password".to_owned()),
            retention_policy: Some("test".to_owned()),
            precision: Some(Precision::Hours),
            consistency: Some(Consistency::All),
        };

        let expected_params = vec![
            ("db", "metrics".to_owned()),
            ("u", "username".to_owned()),
            ("p", "password".to_owned()),
            ("consistency", "all".to_owned()),
            ("precision", "h".to_owned()),
            ("rp", "test".to_owned()),
        ];

        let config = InfluxV1ConfigBuilder::default()
            .endpoint("http://localhost:8086".to_string())
            .db("metrics".to_string())
            .username(Some("username".to_string()))
            .password(Some("password".to_string()))
            .retention_policy(Some("test".to_string()))
            .precision(Some(Precision::Hours))
            .consistency(Some(Consistency::All))
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(expected_config, config);
        assert_eq!(expected_params, config.parameters());
    }
}
