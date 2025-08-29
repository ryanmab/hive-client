use crate::actions::Action;
use crate::{ApiError, Client};

impl Client {
    /// Get all of the [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) setup in the Hive account.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new("Home Automation");
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// client.login(User::new("example@example.com", "example"), trusted_device)
    ///     .await
    ///     .expect("Login should succeed");
    ///
    /// let actions = client.get_actions()
    ///     .await
    ///     .expect("Quick action should be retrieved");
    ///
    /// // Activate a quick action
    /// let mut turn_off_heating = actions.into_iter()
    ///     .find(|action| action.data.id == "1234-5678-000-0000")
    ///     .expect("Quick action to turn off heating should exist");
    ///
    /// let activated = turn_off_heating.activate().await;
    ///
    /// assert!(activated.is_ok());
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the list of [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) could not be retrieved.
    pub async fn get_actions(&self) -> Result<Vec<Action<'_>>, ApiError> {
        self.api
            .get_actions_data(&*self.refresh_tokens_if_needed().await?)
            .await
            .map(|actions| {
                actions
                    .into_iter()
                    .map(|data| Action::new(self, data))
                    .collect()
            })
    }

    /// Activate a Quick Action by a given ID.
    ///
    /// Wrapped by [`Action::activate`] to activate a returned Quick Action.
    pub(crate) async fn activate_action(&self, action_id: &str) -> Result<bool, ApiError> {
        self.api
            .activate_action(&*self.refresh_tokens_if_needed().await?, action_id)
            .await
    }
}
