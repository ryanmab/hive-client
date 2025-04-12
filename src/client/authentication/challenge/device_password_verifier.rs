use crate::authentication::user::AccountDevice;
use crate::authentication::{LoginSession, User};
use crate::constants::CLIENT_ID;
use crate::AuthenticationError;
use aws_cognito_srp::SrpClient;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;
use std::collections::HashMap;

pub async fn handle_challenge(
    user: &User,
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    srp_client: &SrpClient<aws_cognito_srp::TrackedDevice>,
    session: &LoginSession,
    parameters: HashMap<String, String>,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let srp_b = parameters
        .get("SRP_B")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SRP_B".to_string()))?;
    let user_id = parameters
        .get("USERNAME")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("USERNAME".to_string()))?;
    let salt = parameters
        .get("SALT")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SALT".to_string()))?;
    let secret_block = parameters.get("SECRET_BLOCK").ok_or_else(|| {
        AuthenticationError::MissingChallengeParameter("SECRET_BLOCK".to_string())
    })?;

    let parameters = srp_client.verify(secret_block, user_id, salt, srp_b)?;

    let mut builder = cognito_client
        .respond_to_auth_challenge()
        .challenge_name(ChallengeNameType::DevicePasswordVerifier)
        .set_session(session.1.clone())
        .client_id(CLIENT_ID)
        .challenge_responses("USERNAME", session.0.clone())
        .challenge_responses(
            "PASSWORD_CLAIM_SECRET_BLOCK",
            parameters.password_claim_secret_block,
        )
        .challenge_responses(
            "PASSWORD_CLAIM_SIGNATURE",
            parameters.password_claim_signature,
        )
        .challenge_responses("TIMESTAMP", parameters.timestamp);

    if let Some(AccountDevice::Trusted(trusted_device)) = &user.account_device {
        builder = builder.challenge_responses("DEVICE_KEY", &trusted_device.device_key);
    }

    Ok(builder.send().await?)
}
