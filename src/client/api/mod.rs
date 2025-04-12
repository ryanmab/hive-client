mod error;

/// Support for [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) API.
pub mod actions;

/// Support for Hive Devices API ([Thermostat](https://www.hivehome.com/shop/smart-heating/hive-thermostat), [Hive Hub](https://www.hivehome.com/shop/smart-home/hive-hub), Boiler Modules, etc).
pub mod devices;

/// Support for Hive Products API (Heating, Hot Water, etc).
pub mod products;

/// Support for the Hive Weather API.
pub mod weather;

pub use error::ApiError;

#[derive(Debug)]
pub struct HiveApi {
    client: reqwest::Client,
}

impl HiveApi {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}
