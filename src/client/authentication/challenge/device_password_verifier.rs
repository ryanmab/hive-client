use crate::AuthenticationError;
use crate::authentication::LoginSession;
use crate::constants::CLIENT_ID;
use aws_cognito_srp::{SrpClient, VerificationParameters};
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;
use std::collections::HashMap;

pub async fn handle_challenge(
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    device_srp_client: &SrpClient<aws_cognito_srp::TrackedDevice>,
    session: &LoginSession,
    parameters: HashMap<String, String>,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let srp_b = parameters
        .get("SRP_B")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SRP_B".to_string()))?;
    let salt = parameters
        .get("SALT")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SALT".to_string()))?;
    let secret_block = parameters.get("SECRET_BLOCK").ok_or_else(|| {
        AuthenticationError::MissingChallengeParameter("SECRET_BLOCK".to_string())
    })?;

    let VerificationParameters {
        password_claim_secret_block,
        password_claim_signature,
        timestamp,
        ..
    } = device_srp_client.verify(secret_block, salt, srp_b)?;
    let device_key = device_srp_client.get_auth_parameters().device_key;

    Ok(cognito_client
        .respond_to_auth_challenge()
        .challenge_name(ChallengeNameType::DevicePasswordVerifier)
        .set_session(session.1.clone())
        .client_id(CLIENT_ID)
        .challenge_responses("USERNAME", session.0.clone())
        .challenge_responses("PASSWORD_CLAIM_SECRET_BLOCK", password_claim_secret_block)
        .challenge_responses("PASSWORD_CLAIM_SIGNATURE", password_claim_signature)
        .challenge_responses("TIMESTAMP", timestamp)
        .challenge_responses("DEVICE_KEY", device_key)
        .send()
        .await?)
}
