use crate::authentication::user::{AuthDevice, UntrustedDevice};
use crate::client::authentication::TrustedDevice;
use crate::client::authentication::{DeviceClient, HiveAuth, Tokens};
use crate::AuthenticationError;
use aws_cognito_srp::PasswordVerifierParameters;
use aws_sdk_cognitoidentityprovider::operation::confirm_device::ConfirmDeviceOutput;
use aws_sdk_cognitoidentityprovider::types::builders::DeviceSecretVerifierConfigTypeBuilder;
use aws_sdk_cognitoidentityprovider::types::DeviceRememberedStatusType;

impl HiveAuth {
    pub async fn confirm_device(
        &self,
        username: &str,
        device_name: &str,
        untrusted_device: UntrustedDevice,
        tokens: &Tokens,
    ) -> Result<TrustedDevice, AuthenticationError> {
        let device_key = untrusted_device.device_key.clone();
        let device_group_key = untrusted_device.device_group_key.clone();

        let lock = self
            .get_device_srp_client(username, &AuthDevice::Untrusted(untrusted_device))
            .await;

        let srp_client = &mut *lock.write().await;

        let PasswordVerifierParameters {
            verifier: password_verifier,
            salt,
            password,
        } = match srp_client {
            Some(DeviceClient::Untracked(srp_client)) => srp_client.get_password_verifier(),
            _ => unreachable!("The device client must be untracked in order to have reached this point when confirming the device."),
        };

        let response = self
            .cognito
            .confirm_device()
            .device_key(&device_key)
            .device_name(device_name)
            .access_token(&tokens.access_token)
            .device_secret_verifier_config(
                DeviceSecretVerifierConfigTypeBuilder::default()
                    .password_verifier(&password_verifier)
                    .salt(&salt)
                    .build(),
            )
            .send()
            .await
            .map_err(|sdk_error| AuthenticationError::DeviceConfirmationError(sdk_error.into()))?;

        if let ConfirmDeviceOutput {
            user_confirmation_necessary: true,
            ..
        } = response
        {
            // The device wont automatically be confirmed, unless we prompt the user pool
            // to update the state
            self.cognito
                .update_device_status()
                .device_key(&device_key)
                .device_remembered_status(DeviceRememberedStatusType::Remembered)
                .access_token(&tokens.access_token)
                .send()
                .await
                .map_err(|sdk_error| {
                    AuthenticationError::DeviceConfirmationError(sdk_error.into())
                })?;
        }

        // We've confirmed the device now, so we no longer need to device client
        srp_client.take();

        Ok(TrustedDevice::new(
            &password,
            &device_group_key,
            &device_key,
        ))
    }
}
