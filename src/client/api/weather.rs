use crate::client::api::{ApiError, HiveApi};
use crate::client::authentication::Tokens;
use crate::helper::url::{get_base_url, Url};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "unit")]
/// The current weather temperature.
pub enum Temperature {
    #[serde(rename = "C")]
    #[allow(missing_docs)]
    Celsius { value: f32 },
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Celsius { value } => write!(f, "{value}°C"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(missing_docs)]
pub struct WeatherData {
    /// An enumeration of different whether types (i.e. "`clear_sky`").
    #[serde(rename = "icon")]
    pub r#type: String,

    /// The current temperature.
    pub temperature: Temperature,

    /// A human readable description of the weather (i.e. "clear sky").
    pub description: String,
}

/// Weather information returned from Hive.
#[derive(Serialize, Deserialize, Debug)]
pub struct Weather {
    #[allow(missing_docs)]
    #[serde(rename = "weather")]
    pub data: WeatherData,
}

impl HiveApi {
    pub(crate) async fn get_weather(
        &self,
        tokens: &Tokens,
        postcode: &str,
    ) -> Result<Weather, ApiError> {
        let response = self
            .client
            .get(get_base_url(&Url::Weather))
            .query(&[("postcode", postcode.replace(' ', ""))])
            .header("Authorization", &tokens.id_token)
            .send()
            .await;

        Ok(response?.json::<Weather>().await?)
    }
}
