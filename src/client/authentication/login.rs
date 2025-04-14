use crate::authentication::user::UntrustedDevice;
use crate::client::authentication::User;
use crate::client::authentication::{ChallengeResponse, HiveAuth, Tokens};
use crate::{constants, AuthenticationError};
use aws_sdk_cognitoidentityprovider::types::{
    AuthFlowType, AuthenticationResultType, ChallengeNameType, NewDeviceMetadataType,
};

#[derive(Debug, Default)]
pub struct LoginSession(pub String, pub Option<String>);

impl HiveAuth {
    /// Login to the Hive API using the provided user credentials.
    ///
    /// The user may, or may not, have a device associated with their account. However, if they do,
    /// this will allow the authentication process to complete without the need to capture a 2FA code.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication fails, or if the user is not registered with the Hive API.
    pub async fn login(
        &self,
        user: &User,
    ) -> Result<(Tokens, Option<UntrustedDevice>), AuthenticationError> {
        let parameters = self
            .get_user_srp_client(user)
            .await
            .read()
            .await
            .as_ref()
            .unwrap()
            .get_auth_parameters();

        let mut builder = self
            .cognito
            .initiate_auth()
            .auth_flow(AuthFlowType::UserSrpAuth)
            .client_id(constants::CLIENT_ID)
            .auth_parameters("SRP_A", parameters.a)
            .auth_parameters("USERNAME", &user.username);

        if let Some(device_key) = parameters.device_key {
            builder = builder.auth_parameters("DEVICE_KEY", device_key);
        }

        let response = builder.send().await?;

        {
            self.session.write().await.replace(LoginSession(
                user.username.as_str().to_string(),
                response.session,
            ));
        }

        match response.challenge_name {
            None => {
                if let Some(AuthenticationResultType {
                    id_token: Some(id_token),
                    access_token: Some(access_token),
                    refresh_token: Some(refresh_token),
                    expires_in,
                    new_device_metadata,
                    ..
                }) = response.authentication_result
                {
                    let mut untrusted_device: Option<UntrustedDevice> = None;

                    if let Some(NewDeviceMetadataType {
                        device_key: Some(device_key),
                        device_group_key: Some(device_group_key),
                        ..
                    }) = new_device_metadata
                    {
                        untrusted_device =
                            Some(UntrustedDevice::new(&device_group_key, &device_key));
                    }

                    Ok((
                        Tokens::new(id_token, access_token, refresh_token, expires_in),
                        untrusted_device,
                    ))
                } else {
                    Err(AuthenticationError::AccessTokenNotValid)
                }
            }
            Some(ChallengeNameType::PasswordVerifier) => {
                self.respond_to_challenge(
                    user,
                    ChallengeResponse::PasswordVerifier(
                        response.challenge_parameters.unwrap_or_default(),
                    ),
                )
                .await
            }
            Some(name) => Err(AuthenticationError::UnsupportedChallenge(name.to_string())),
        }
    }
}
