use crate::constants;
use aws_cognito_srp::{SrpClient, TrackedDevice, UntrackedDevice};
use aws_config::BehaviorVersion;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::RwLock;

mod challenge;
mod confirm_device;
mod error;
mod login;
mod logout;
mod refresh;
mod user;

pub use challenge::{ChallengeRequest, ChallengeResponse};
pub use error::{AuthenticationError, DeviceConfirmationError};
pub use user::{TrustedDevice, User};

pub(crate) use login::LoginSession;
pub(crate) use user::AuthDevice;
pub(crate) use user::Tokens;

#[derive(Debug)]
enum DeviceClient {
    Tracked(SrpClient<TrackedDevice>),
    Untracked(SrpClient<UntrackedDevice>),
}

#[derive(Default, Debug)]
struct SrpClients {
    user: Arc<RwLock<Option<SrpClient<aws_cognito_srp::User>>>>,
    device: Arc<RwLock<Option<DeviceClient>>>,
}

#[derive(Debug)]
pub(crate) struct HiveAuth {
    cognito: aws_sdk_cognitoidentityprovider::Client,
    srp: SrpClients,
    session: Arc<RwLock<Option<LoginSession>>>,
}

impl HiveAuth {
    #[must_use]
    pub(crate) async fn new() -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(constants::REGION)
            .load()
            .await;

        Self {
            cognito: aws_sdk_cognitoidentityprovider::Client::new(&config),
            srp: SrpClients::default(),
            session: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_user_srp_client(
        &self,
        user: &User,
    ) -> Arc<RwLock<Option<SrpClient<aws_cognito_srp::User>>>> {
        let credentials =
            aws_cognito_srp::User::new(constants::POOL_ID, &user.username, &user.password);

        let client = Arc::clone(&self.srp.user);

        {
            let mut client = client.write().await;

            if let Some(client) = &mut *client {
                client.replace_credentials(credentials);
            } else {
                client.replace(SrpClient::new(credentials, constants::CLIENT_ID, None));
            }
        }

        client
    }

    async fn get_device_srp_client(
        &self,
        username: &str,
        account_device: &AuthDevice,
    ) -> Arc<RwLock<Option<DeviceClient>>> {
        let client = Arc::clone(&self.srp.device);

        {
            let mut client = client.write().await;

            match &account_device {
                AuthDevice::Untrusted(untrusted_device) => {
                    let credentials = UntrackedDevice::new(
                        constants::POOL_ID,
                        &untrusted_device.device_group_key,
                        &untrusted_device.device_key,
                    );

                    match &mut *client {
                        Some(DeviceClient::Untracked(client)) => {
                            client.replace_credentials(credentials);
                        }
                        Some(DeviceClient::Tracked(_)) | None => {
                            client.replace(DeviceClient::Untracked(SrpClient::new(
                                credentials,
                                constants::CLIENT_ID,
                                None,
                            )));
                        }
                    }
                }
                AuthDevice::Trusted(trusted_device) => {
                    let credentials = TrackedDevice::new(
                        constants::POOL_ID,
                        username,
                        &trusted_device.device_group_key,
                        &trusted_device.device_key,
                        &trusted_device.device_password,
                    );

                    if let Some(DeviceClient::Untracked(old_client)) =
                        client.take_if(|c| matches!(c, DeviceClient::Untracked(_)))
                    {
                        let _ = client.insert(DeviceClient::Tracked(SrpClient::new(
                            old_client
                                .take_credentials()
                                .into_tracked(username, &trusted_device.device_password),
                            constants::CLIENT_ID,
                            None,
                        )));
                    } else if let Some(DeviceClient::Tracked(client)) = &mut *client {
                        client.replace_credentials(credentials);
                    } else if client.deref_mut().is_none() {
                        client.replace(DeviceClient::Tracked(SrpClient::new(
                            credentials,
                            constants::CLIENT_ID,
                            None,
                        )));
                    }
                }
            }
        }

        client
    }
}
