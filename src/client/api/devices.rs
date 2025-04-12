use std::collections::HashMap;

use crate::client::api::error::ApiError;
use crate::client::api::HiveApi;
use crate::client::authentication::Tokens;
use crate::helper::url::{get_base_url, Url};
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum PowerType {
    /// The device is powered by an internal battery.
    Battery,

    /// The device is connected directly to the mains power supply.
    Mains,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct Properties {
    #[serde(rename = "online")]
    /// Whether the device is currently online or not.
    pub is_online: bool,

    /// The type of power source used by the device (if applicable).
    pub power: Option<PowerType>,

    #[serde(rename = "battery")]
    /// The battery percentage of the device (if applicable).
    pub battery_percentage: Option<i32>,

    #[serde(rename = "zone")]
    /// The ID of the zone the device is located in (if applicable).
    pub zone_id: Option<String>,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct State {
    /// The name of the device.
    pub name: String,

    /// The name of the zone the device is located in (if applicable).
    pub zone_name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A [Hive Thermostat](https://www.hivehome.com/shop/smart-heating/hive-thermostat).
pub struct Thermostat {
    /// The unique ID of the Thermostat.
    pub id: String,

    #[serde(with = "ts_milliseconds")]
    /// The date and time the Thermostat last communicated with the Hub.
    pub last_seen: DateTime<Utc>,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Thermostat was first created.
    pub created_at: DateTime<Utc>,

    #[serde(rename = "props")]
    /// The properties of the Thermostat.
    pub properties: Properties,

    /// The current state of the Thermostat.
    pub state: State,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A [Hive Hub](https://www.hivehome.com/shop/smart-home/hive-hub).
pub struct Hub {
    /// The unique ID of the Hub.
    pub id: String,

    #[serde(with = "ts_milliseconds")]
    /// The date and time the Hub last communicated with the Hive servers.
    pub last_seen: DateTime<Utc>,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Hub was first created.
    pub created_at: DateTime<Utc>,

    #[serde(rename = "props")]
    /// The properties of the Hub.
    pub properties: Properties,

    /// The current state of the Hub.
    pub state: State,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A Hive Boiler Module.
pub struct BoilerModule {
    /// The unique ID of the Boiler Module.
    pub id: String,

    #[serde(with = "ts_milliseconds")]
    /// The date and time the Boiler Module last communicated with the Hub.
    pub last_seen: DateTime<Utc>,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Boiler Module was first created.
    pub created_at: DateTime<Utc>,

    #[serde(rename = "props")]
    /// The properties of the Boiler Module.
    pub properties: Properties,

    /// The current state of the Boiler Module.
    pub state: State,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum DeviceData {
    #[serde(rename = "thermostatui")]
    /// A [Hive Thermostat](https://www.hivehome.com/shop/smart-heating/hive-thermostat).
    Thermostat(Thermostat),

    /// A [Hive Hub](https://www.hivehome.com/shop/smart-home/hive-hub).
    Hub(Hub),

    /// A Hive Boiler Module.
    BoilerModule(BoilerModule),

    #[serde(other)]
    /// A device which is yet to be mapped by the crate.
    Unknown,
}

/// A Device setup in a Hive account.
///
/// For example, a [`DeviceData::Thermostat`], a [`DeviceData::Hub`], etc.
#[derive(Debug)]
pub struct Device {
    #[allow(missing_docs)]
    pub data: DeviceData,
}

impl Device {
    pub(crate) const fn new(data: DeviceData) -> Self {
        Self { data }
    }
}

impl HiveApi {
    pub(crate) async fn get_devices(&self, tokens: &Tokens) -> Result<Vec<DeviceData>, ApiError> {
        let response = self
            .client
            .get(get_base_url(&Url::Device))
            .header("Authorization", &tokens.id_token)
            .send()
            .await;

        let body = response?.text().await?;

        Ok(serde_json::from_str(&body)?)
    }
}
