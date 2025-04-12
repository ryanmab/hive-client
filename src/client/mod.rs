mod api;
mod wrapper;

/// Support for the Hive Authentication API.
pub mod authentication;

pub use api::actions;
pub use api::devices;
pub use api::products;
pub use api::weather;

pub use api::ApiError;
pub use authentication::AuthenticationError;

use crate::authentication::HiveAuth;
use crate::client::api::HiveApi;
use crate::client::authentication::{Tokens, User};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Client used to authenticate and interact with Hive.
#[derive(Debug)]
pub struct Client {
    auth: HiveAuth,
    api: HiveApi,
    user: Mutex<Option<User>>,
    tokens: Mutex<Option<Arc<Tokens>>>,
}

impl Client {
    #[allow(missing_docs)]
    pub async fn new() -> Self {
        Self {
            auth: HiveAuth::new().await,
            api: HiveApi::new(),
            user: Mutex::new(None),
            tokens: Mutex::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy_macro::dotenv;

    #[tokio::test]
    async fn test_cognito_authentication_and_device_confirmation() {
        let client = Client::new().await;

        let user = User::new(
            dotenv!("MOCK_USER_EMAIL"),
            dotenv!("MOCK_USER_PASSWORD"),
            None,
        );

        client.login(user).await.expect("Login should succeed");

        let trusted_device = client
            .confirm_device("mock-device")
            .await
            .expect("Confirm device should succeed");

        assert!(!trusted_device.device_key.is_empty());
        assert!(!trusted_device.device_group_key.is_empty());
        assert!(!trusted_device.device_password.is_empty());
        assert!(trusted_device.device_key.starts_with(dotenv!("REGION")));
    }
}
