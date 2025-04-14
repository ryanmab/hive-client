use crate::devices::Device;
use crate::{ApiError, Client};

impl Client {
    /// Get all of the devices associated with the Hive account.
    ///
    /// This can include Hubs, Thermostats, Boilers, and other devices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    /// use hive_client::products::{Product, ProductData, State, States};
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new("Home Automation").await;
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", trusted_device)).await;
    ///
    /// if let Ok(_) = attempt {
    ///     // Login was successful
    ///
    ///     let devices = client.get_devices()
    ///         .await
    ///         .expect("Devices should be retrieved");
    ///     
    ///     println!("{:?}", devices);
    /// }
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the list of devices could not be retrieved.
    pub async fn get_devices(&self) -> Result<Vec<Device>, ApiError> {
        self.api
            .get_devices(&*self.refresh_tokens_if_needed().await?)
            .await
            .map(|data| data.into_iter().map(Device::new).collect())
    }
}
