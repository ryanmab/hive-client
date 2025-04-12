use crate::weather::Weather;
use crate::{ApiError, Client};

impl Client {
    /// Get the current weather according to Hive, for a given postcode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    /// use hive_client::{weather::WeatherData};
    /// use hive_client::weather::Temperature::Celsius;
    /// use hive_client::weather::Weather;
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new().await;
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// client.login(User::new("example@example.com", "example", trusted_device))
    ///     .await
    ///     .expect("Login should succeed");
    ///
    /// let Weather { data: WeatherData { temperature, ..}} = client.get_weather("SW1A 1AA")
    ///     .await
    ///     .expect("Weather should be retrieved");
    ///
    /// println!("The current temperature is: {temperature}");
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the whether data could not be retrieved.
    pub async fn get_weather(&self, postcode: &str) -> Result<Weather, ApiError> {
        self.api
            .get_weather(&*self.refresh_tokens_if_needed().await?, postcode)
            .await
    }
}
