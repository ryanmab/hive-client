use crate::client::api::error::ApiError;
use crate::client::api::HiveApi;
use crate::client::authentication::Tokens;
use crate::helper::url::{get_base_url, Url};
use crate::Client;
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Deserialize, Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct ActionData {
    /// The unique ID of the Quick Action.
    pub id: String,

    /// The name of the Quick Action.
    pub name: String,

    /// Whether the Quick Action is enabled or not.
    pub enabled: bool,

    /// The template used for the Quick Action.
    pub template: String,

    #[serde(with = "ts_milliseconds")]
    #[serde(rename = "created")]
    /// The date and time when the Quick Action was first created.
    pub created_at: DateTime<Utc>,

    #[serde(flatten)]
    #[allow(missing_docs)]
    pub extra: HashMap<String, Value>,
}

/// A [Quick Action](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) setup in the Hive account.
pub struct Action<'a> {
    client: &'a Client,

    #[allow(missing_docs)]
    pub data: ActionData,
}

impl Debug for Action<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action").field("data", &self.data).finish()
    }
}

impl Action<'_> {
    #[must_use]
    pub(crate) const fn new(client: &Client, data: ActionData) -> Action<'_> {
        Action { client, data }
    }

    /// Activate the [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new("Home Automation").await;
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
    /// Returns an error if the [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) could not be activated.
    pub async fn activate(&mut self) -> Result<bool, ApiError> {
        self.client.activate_action(&self.data.id).await
    }
}

impl HiveApi {
    pub(crate) async fn get_actions_data(
        &self,
        tokens: &Tokens,
    ) -> Result<Vec<ActionData>, ApiError> {
        let response = self
            .client
            .get(get_base_url(&Url::Actions {
                id: None,
                activate: false,
            }))
            .header("Authorization", &tokens.id_token)
            .send()
            .await;

        response?
            .json::<Vec<ActionData>>()
            .await
            .map_err(ApiError::from)
    }

    pub(crate) async fn activate_action(
        &self,
        tokens: &Tokens,
        action_id: &str,
    ) -> Result<bool, ApiError> {
        let response = self
            .client
            .post(get_base_url(&Url::Actions {
                id: Some(action_id),
                activate: true,
            }))
            .body("{}")
            .header("Authorization", &tokens.id_token)
            .send()
            .await?;

        Ok(response.status() == StatusCode::OK)
    }
}
