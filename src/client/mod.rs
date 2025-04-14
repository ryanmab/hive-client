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
    friendly_name: String,
}

impl Client {
    /// Create a new client.
    ///
    /// The friendly name is used to identify the client in the
    /// [Trusted Device page](https://community.hivehome.com/s/article/2FA-2-factor-Authentication) of the Hive app if
    /// the user is authenticating for the first time (does not have a trusted device during [`Client::login`])
    pub async fn new(friendly_name: &str) -> Self {
        Self {
            auth: HiveAuth::new().await,
            api: HiveApi::new(),
            user: Mutex::new(None),
            tokens: Mutex::new(None),
            friendly_name: friendly_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy_macro::dotenv;

    #[tokio::test]
    async fn test_cognito_authentication_and_device_confirmation() {
        let mut client = Client::new("Home Automation").await;

        let user = User::new(
            dotenv!("MOCK_USER_EMAIL"),
            dotenv!("MOCK_USER_PASSWORD"),
            None,
        );

        let trusted_device = client
            .login(user)
            .await
            .expect("Login should succeed")
            .expect("A trusted device should've been returned");

        assert!(!trusted_device.device_key.is_empty());
        assert!(!trusted_device.device_group_key.is_empty());
        assert!(!trusted_device.device_password.is_empty());
        assert!(trusted_device.device_key.starts_with(dotenv!("REGION")));

        client.logout().await.expect("Logout should succeed");
    }

    #[tokio::test]
    async fn test_cognito_authentication_refresh() {
        let client = Client::new("Home Automation").await;

        let user = User::new(
            dotenv!("MOCK_USER_EMAIL"),
            dotenv!("MOCK_USER_PASSWORD"),
            None,
        );

        client.login(user).await.expect("Login should succeed");

        let current_tokens = {
            // Update the tokens to simulate an expiration

            let mut tokens = client.tokens.lock().await;

            let current_tokens = tokens.clone().unwrap();

            let replacement_tokens = Arc::new(Tokens::new(
                current_tokens.id_token.to_string(),
                current_tokens.access_token.to_string(),
                current_tokens.refresh_token.to_string(),
                0,
            ));
            tokens.replace(Arc::clone(&replacement_tokens));

            replacement_tokens
        };

        let refreshed_tokens = client
            .refresh_tokens_if_needed()
            .await
            .expect("Refresh tokens should succeed");

        assert_ne!(current_tokens.id_token, refreshed_tokens.id_token);
        assert_ne!(current_tokens.access_token, refreshed_tokens.access_token);
        assert_eq!(current_tokens.refresh_token, refreshed_tokens.refresh_token);
        assert!(current_tokens.expires_at < refreshed_tokens.expires_at);
    }
}
