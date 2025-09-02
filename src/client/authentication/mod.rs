use crate::constants;
use aws_cognito_srp::{SrpClient, TrackedDevice};
use aws_config::BehaviorVersion;
use std::sync::Arc;
use tokio::sync::RwLock;

mod challenge;
mod confirm_device;
mod error;
mod login;
mod refresh;
mod user;

pub use challenge::{ChallengeRequest, ChallengeResponse};
pub use error::{AuthenticationError, DeviceConfirmationError};
pub use user::{TrustedDevice, User};

pub(crate) use login::LoginSession;
pub(crate) use user::{Tokens, UntrustedDevice};

#[derive(Debug)]
pub(crate) struct HiveAuth {
    cognito: aws_sdk_cognitoidentityprovider::Client,
    user_srp_client: SrpClient<aws_cognito_srp::User>,
    device_srp_client: Option<SrpClient<TrackedDevice>>,
    session: Arc<RwLock<Option<LoginSession>>>,
}

impl HiveAuth {
    #[must_use]
    pub(crate) async fn new(user: &User, trusted_device: Option<&TrustedDevice>) -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(constants::REGION)
            .load()
            .await;

        let auth = Self {
            cognito: aws_sdk_cognitoidentityprovider::Client::new(&config),
            user_srp_client: SrpClient::new(
                aws_cognito_srp::User::new(constants::POOL_ID, &user.username, &user.password),
                constants::CLIENT_ID,
                None,
            ),
            device_srp_client: None,
            session: Arc::new(RwLock::new(None)),
        };

        auth.replace_trusted_device(trusted_device);

        auth
    }

    pub(crate) fn replace_trusted_device(&mut self, trusted_device: Option<&TrustedDevice>) {
        self.device_srp_client = trusted_device.map(|trusted_device| {
            SrpClient::new(
                TrackedDevice::new(
                    constants::POOL_ID,
                    &trusted_device.device_group_key,
                    &trusted_device.device_key,
                    &trusted_device.device_password,
                ),
                constants::CLIENT_ID,
                None,
            )
        });
    }
}
