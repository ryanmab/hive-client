use crate::client::api::ApiError;
use crate::client::api::HiveApi;
use crate::client::authentication::Tokens;
use crate::helper::url::{get_base_url, Url};
use crate::Client;
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, EnumMap};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct Properties {
    #[serde(rename = "zone")]
    /// The ID of the zone the device is located in (if applicable).
    pub zone_id: Option<String>,

    #[serde(rename = "online")]
    /// Whether the device is currently online or not.
    pub is_online: bool,

    #[serde(rename = "working")]
    /// Whether the device is currently running or not.
    pub is_working: bool,

    /// The current temperature by the Hive product.
    pub temperature: Option<f32>,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
#[serde_as]
#[allow(missing_docs)]
pub struct Heating {
    /// The unique ID of the Hive Heating product.
    pub id: String,

    #[serde(with = "ts_milliseconds")]
    /// The date and time when the Hive Heating product last communicated with the Hive servers.
    pub last_seen: DateTime<Utc>,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Hive Heating product was first created.
    pub created_at: DateTime<Utc>,

    #[serde(rename = "props")]
    /// The properties of the Hive Heating product.
    pub properties: Properties,

    /// The current state of the Hive Heating product.
    pub state: States,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct HotWater {
    /// The unique ID of the Hive Hot Water product.
    pub id: String,

    #[serde(with = "ts_milliseconds")]
    /// The date and time when the Hive Hot Water product last communicated with the Hive servers.
    pub last_seen: DateTime<Utc>,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Hive Hot Water product was first created.
    pub created_at: DateTime<Utc>,

    #[serde(rename = "props")]
    /// The properties of the Hive Hot Water product.
    pub properties: Properties,

    /// The current state of the Hive Hot Water product.
    pub state: States,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
#[non_exhaustive]
/// Data about a Hive product.
pub enum ProductData {
    /// A Hive Heating product.
    Heating(Heating),

    /// A Hive Hot Water product.
    HotWater(HotWater),

    #[serde(other)]
    /// A product which is yet to be mapped by the crate.
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
/// The mode of a Hive product.
///
/// This applies to both [`ProductData::Heating`] and [`ProductData::HotWater`], which can be
/// either in `Off`, `Schedule` or `Manual` mode.
pub enum Mode {
    /// The product is turned off.
    Off,

    /// The product is in schedule mode.
    Schedule,

    /// The product is in manual mode.
    Manual,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Schedule => write!(f, "Schedule"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// The state of a particular facet of a Hive product.
///
/// Not all products will have all states - for example [`ProductData::HotWater`] will not have
/// [`State::TargetTemperature`]
pub enum State {
    #[serde(rename = "target")]
    /// The target temperature of the Hive product.
    TargetTemperature(f32),

    /// The mode of the Hive product.
    Mode(Mode),

    /// The name of the Hive product.
    Name(String),

    /// The status of the Hive product.
    Status(String),

    /// Whether the Hive product is currently boosted or not.
    Boost(Option<bool>),

    /// The temperature of the Frost Protection mode.
    FrostProtection(u32),

    /// Whether the Hive product will choose an Optimum Start time or not when
    /// in scheduled mode.
    OptimumStart(bool),

    /// Whether the Hive product is currently in Auto Boost mode or not.
    AutoBoost(String),

    /// The target temperature of the Auto Boost mode.
    AutoBoostTarget(u32),

    /// The schedule for the Hive product, when it is in [`Mode::Schedule`].
    Schedule(HashMap<String, Value>),
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetTemperature(temp) => write!(f, "{temp}"),
            Self::Mode(value) => write!(f, "{value}"),
            Self::Name(value) | Self::Status(value) | Self::AutoBoost(value) => {
                write!(f, "{value}")
            }
            Self::Boost(value) => write!(f, "{value:?}"),
            Self::FrostProtection(value) | Self::AutoBoostTarget(value) => write!(f, "{value}"),
            Self::OptimumStart(value) => write!(f, "{value}"),
            Self::Schedule(value) => write!(f, "{value:?}"),
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
/// A collection of states for a Hive product.
pub struct States(#[serde_as(as = "EnumMap")] pub Vec<State>);

impl Deref for States {
    type Target = Vec<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Product which is enabled in a Hive account.
///
/// For example, a [`ProductData::Heating`], a [`ProductData::HotWater`], etc.
pub struct Product<'a> {
    client: &'a Client,

    #[allow(missing_docs)]
    pub data: ProductData,
}

impl Debug for Product<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Product").field("data", &self.data).finish()
    }
}

impl Product<'_> {
    #[must_use]
    pub(crate) const fn new(client: &Client, data: ProductData) -> Product<'_> {
        Product { client, data }
    }

    /// Set the state of a product.
    ///
    /// For example, setting the target temperature of the Heating product, set the mode
    /// ([`crate::products::State::Mode`]) of a Hot Water product, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if the state could not be set for the product.
    pub async fn set_state(&mut self, states: States) -> Result<bool, ApiError> {
        self.client
            .set_product_state(
                match &self.data {
                    ProductData::Heating(data) => &data.id,
                    ProductData::HotWater(data) => &data.id,
                    ProductData::Unknown => "",
                },
                match &self.data {
                    ProductData::Heating(_) => "heating",
                    ProductData::HotWater(_) => "hotwater",
                    ProductData::Unknown => "unknown",
                },
                states,
            )
            .await
    }
}

impl HiveApi {
    pub(crate) async fn get_product_data(
        &self,
        tokens: &Tokens,
    ) -> Result<Vec<ProductData>, ApiError> {
        let response = self
            .client
            .get(get_base_url(&Url::Products))
            .header("Authorization", &tokens.id_token)
            .send()
            .await;

        response?
            .json::<Vec<ProductData>>()
            .await
            .map_err(ApiError::from)
    }

    pub(crate) async fn set_product_state(
        &self,
        tokens: &Tokens,
        id: &str,
        r#type: &str,
        states: States,
    ) -> Result<bool, ApiError> {
        let response = self
            .client
            .post(get_base_url(&Url::Node {
                id: Some(id),
                r#type: Some(r#type),
            }))
            .body(serde_json::to_string(&states)?)
            .header("Authorization", &tokens.id_token)
            .send()
            .await?;

        Ok(response.status() == StatusCode::OK)
    }
}
