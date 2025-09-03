use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::ops::Add;

#[derive(Debug)]
/// A user registed with a Hive account.
pub struct User {
    /// The username of the user - this is the email address used
    /// to register the account.
    pub(crate) username: String,
    pub(crate) password: String,
}

impl User {
    #[must_use]
    /// Create a new user with the given username and password.
    ///
    /// Optionally, a trusted device can be provided which will be used to authenticate the user without
    /// the need to go through additional [`crate::authentication::ChallengeRequest`]s - like SMS MFA.
    pub fn new<'a>(username: &'a str, password: &'a str) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// A trusted device is a device that has been confirmed.
///
/// Hive uses AWS Cognito for authentication, and a trusted device in a Hive account is actually a tracked
/// device in AWS Cognito.
///
/// See the [AWS Cognito documentation](https://docs.aws.amazon.com/cognito/latest/developerguide/amazon-cognito-user-pools-device-tracking.html#user-pools-remembered-devices-getting-a-device-key) for more information.
#[derive(Debug, Clone)]
pub struct TrustedDevice {
    #[allow(missing_docs)]
    pub device_group_key: String,

    #[allow(missing_docs)]
    pub device_key: String,

    #[allow(missing_docs)]
    pub device_password: String,
}

impl TrustedDevice {
    #[must_use]
    /// Create a new trusted device which can be used to authenticate the user.
    ///
    /// ```rust
    /// use hive_client::authentication::{TrustedDevice};
    ///
    /// // Create the trusted device with the device password, group key and key.
    /// let trusted_device = TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// );
    /// ```
    pub fn new<'a>(
        device_password: &'a str,
        device_group_key: &'a str,
        device_key: &'a str,
    ) -> Self {
        Self {
            device_password: device_password.into(),
            device_group_key: device_group_key.into(),
            device_key: device_key.into(),
        }
    }
}

#[derive(Debug)]
pub struct UntrustedDevice {
    pub device_group_key: String,
    pub device_key: String,
}

impl UntrustedDevice {
    #[must_use]
    pub(crate) fn new<'a>(device_group_key: &'a str, device_key: &'a str) -> Self {
        Self {
            device_group_key: device_group_key.into(),
            device_key: device_key.into(),
        }
    }
}

#[derive(Debug)]
pub struct Tokens {
    pub(crate) id_token: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) expires_at: DateTime<Utc>,
}

impl Tokens {
    #[must_use]
    pub fn new(
        id_token: String,
        access_token: String,
        refresh_token: String,
        expires_in: i32,
    ) -> Self {
        Self {
            id_token,
            access_token,
            refresh_token,
            expires_at: Utc::now().add(chrono::Duration::seconds(i64::from(expires_in))),
        }
    }
}
