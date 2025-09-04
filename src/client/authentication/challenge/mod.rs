use crate::authentication::user::UntrustedDevice;
use crate::client::authentication::{HiveAuth, Tokens};
use crate::AuthenticationError;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::{
    AuthenticationResultType, ChallengeNameType, NewDeviceMetadataType,
};
use std::collections::HashMap;
use std::fmt::Debug;

mod device_password_verifier;
mod device_srp_auth;
mod password_verifier;
mod sms_mfa;

#[derive(Debug)]
#[non_exhaustive]
/// The Hive authentication servers have requested a challenge be responded to before
/// the authentication can be completed.
pub enum ChallengeRequest {
    /// A SMS MFA code has been sent to the user's phone number, and the user must enter it
    /// to continue the authentication flow.
    ///
    /// These codes are sent to the phone number associated with the user account, and will
    /// be six digits long.
    SmsMfa,

    /// The authentication flow has requested a password verifier challenge.
    ///
    /// This will be handled transparently by the crate.
    #[doc(hidden)]
    PasswordVerifier,

    /// The authentication flow has requested an unexpected challenge which cannot be handled by
    /// the crate.
    Unsupported(String),
}

#[derive(Debug)]
#[non_exhaustive]
/// A response to a [`ChallengeRequest`] issued by the Hive authentication servers.
pub enum ChallengeResponse {
    /// A response to the [`ChallengeRequest::SmsMfa`] challenge, with the SMS code delivered to
    /// the user's phone.
    SmsMfa(String),
    #[doc(hidden)]
    PasswordVerifier(HashMap<String, String>),
    #[doc(hidden)]
    DeviceSrpAuth,
    #[doc(hidden)]
    DevicePasswordVerifier(HashMap<String, String>),
}

impl HiveAuth {
    pub(crate) async fn respond_to_challenge(
        &self,
        challenge_response: ChallengeResponse,
    ) -> Result<(Tokens, Option<UntrustedDevice>), AuthenticationError> {
        let response = {
            let mut session = self.session.write().await;
            let session = session
                .as_mut()
                .ok_or(AuthenticationError::NoAuthenticationInProgress)?;

            log::info!(
                "Responding to challenge with response: {:?}",
                &challenge_response
            );

            let response = match challenge_response {
                ChallengeResponse::PasswordVerifier(parameters) => {
                    password_verifier::respond_to_challenge(
                        &self.cognito,
                        &self.user_srp_client,
                        self.device_srp_client.as_ref(),
                        session,
                        parameters,
                    )
                    .await?
                }
                ChallengeResponse::DeviceSrpAuth => {
                    device_srp_auth::handle_challenge(
                        &self.cognito,
                        self.device_srp_client
                            .as_ref()
                            .ok_or(AuthenticationError::NoAuthenticationInProgress)?,
                        session,
                    )
                    .await?
                }
                ChallengeResponse::DevicePasswordVerifier(parameters) => {
                    device_password_verifier::handle_challenge(
                        &self.cognito,
                        self.device_srp_client
                            .as_ref()
                            .ok_or(AuthenticationError::NoAuthenticationInProgress)?,
                        session,
                        parameters,
                    )
                    .await?
                }
                ChallengeResponse::SmsMfa(code) => {
                    sms_mfa::handle_challenge(
                        &self.cognito,
                        self.device_srp_client.as_ref(),
                        session,
                        &code,
                    )
                    .await?
                }
            };

            // Update the session ID so that any subsequent calls are following the flow of the authentication
            // challenges.
            session.1.clone_from(&response.session);

            response
        };

        self.handle_challenge_response(response).await
    }

    async fn handle_challenge_response(
        &self,
        response: RespondToAuthChallengeOutput,
    ) -> Result<(Tokens, Option<UntrustedDevice>), AuthenticationError> {
        match &response.challenge_name {
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
                    Err(AuthenticationError::InvalidAccessToken)
                }
            }
            Some(ChallengeNameType::DeviceSrpAuth) => {
                Box::pin(self.respond_to_challenge(ChallengeResponse::DeviceSrpAuth)).await
            }
            Some(ChallengeNameType::DevicePasswordVerifier) => {
                Box::pin(
                    self.respond_to_challenge(ChallengeResponse::DevicePasswordVerifier(
                        response.challenge_parameters.unwrap_or_default(),
                    )),
                )
                .await
            }
            Some(ChallengeNameType::SmsMfa) => {
                Err(AuthenticationError::NextChallenge(ChallengeRequest::SmsMfa))
            }
            Some(name) => Err(AuthenticationError::UnsupportedChallenge(name.to_string())),
        }
    }
}
