use crate::authentication::user::UntrustedDevice;
use crate::client::authentication::TrustedDevice;
use crate::client::authentication::{HiveAuth, Tokens};
use crate::{constants, AuthenticationError};
use aws_cognito_srp::{PasswordVerifierParameters, SrpClient};
use aws_sdk_cognitoidentityprovider::operation::confirm_device::ConfirmDeviceOutput;
use aws_sdk_cognitoidentityprovider::types::builders::DeviceSecretVerifierConfigTypeBuilder;
use aws_sdk_cognitoidentityprovider::types::DeviceRememberedStatusType;

impl HiveAuth {
    pub async fn confirm_device(
        &self,
        device_name: &str,
        untrusted_device: UntrustedDevice,
        tokens: &Tokens,
    ) -> Result<TrustedDevice, AuthenticationError> {
        let device_key = untrusted_device.device_key.clone();
        let device_group_key = untrusted_device.device_group_key.clone();

        let srp_client = SrpClient::new(
            aws_cognito_srp::UntrackedDevice::new(
                constants::POOL_ID,
                &untrusted_device.device_group_key,
                &untrusted_device.device_key,
            ),
            constants::CLIENT_ID,
            None,
        );

        let PasswordVerifierParameters {
            verifier: password_verifier,
            salt,
            password,
        } = srp_client.get_password_verifier();

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

        Ok(TrustedDevice::new(
            &password,
            &device_group_key,
            &device_key,
        ))
    }
}
