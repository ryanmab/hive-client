use crate::client::authentication::{HiveAuth, Tokens};
use crate::{constants, AuthenticationError};
use aws_sdk_cognitoidentityprovider::operation::initiate_auth::InitiateAuthOutput;
use aws_sdk_cognitoidentityprovider::types::{AuthFlowType, AuthenticationResultType};
use std::sync::Arc;

impl HiveAuth {
    pub async fn refresh_tokens(&self, tokens: Arc<Tokens>) -> Result<Tokens, AuthenticationError> {
        let mut builder = self
            .cognito
            .initiate_auth()
            .client_id(constants::CLIENT_ID)
            .auth_flow(AuthFlowType::RefreshTokenAuth)
            .auth_parameters("REFRESH_TOKEN", &tokens.refresh_token);

        if let Some(device_key) = self
            .device_srp_client
            .as_ref()
            .map(|device_srp_client| device_srp_client.get_auth_parameters().device_key)
        {
            builder = builder.auth_parameters("DEVICE_KEY", device_key);
        }

        let response = builder.send().await?;

        if let InitiateAuthOutput {
            authentication_result:
                Some(AuthenticationResultType {
                    expires_in,
                    id_token: Some(id_token),
                    access_token: Some(access_token),
                    ..
                }),
            ..
        } = response
        {
            log::info!("New set of tokens generated successfully.");

            Ok(Tokens::new(
                id_token,
                access_token,
                tokens.refresh_token.clone(),
                expires_in,
            ))
        } else {
            log::error!("Refresh token request failed.");

            Err(AuthenticationError::AccessTokenNotValid)
        }
    }
}
