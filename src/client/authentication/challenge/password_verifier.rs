use crate::authentication::{LoginSession, User};
use crate::constants::CLIENT_ID;
use crate::AuthenticationError;
use aws_cognito_srp::SrpClient;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;
use std::collections::HashMap;

pub async fn respond_to_challenge(
    user: &User,
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    srp_client: &SrpClient<aws_cognito_srp::User>,
    session: &mut LoginSession,
    parameters: HashMap<String, String>,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let secret_block = parameters.get("SECRET_BLOCK").ok_or_else(|| {
        AuthenticationError::MissingChallengeParameter("SECRET_BLOCK".to_string())
    })?;
    let user_id = parameters.get("USER_ID_FOR_SRP").ok_or_else(|| {
        AuthenticationError::MissingChallengeParameter("USER_ID_FOR_SRP".to_string())
    })?;
    let salt = parameters
        .get("SALT")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SALT".to_string()))?;
    let srp_b = parameters
        .get("SRP_B")
        .ok_or_else(|| AuthenticationError::MissingChallengeParameter("SRP_B".to_string()))?;

    // Its very important to record the user id here, as, although authentication will
    // succeed without using the user id for the SRP, the device confirmation will fail silently
    // with an "Invalid device key" error if the user id is not used.
    //
    // See: https://repost.aws/questions/QU3hWYIXPnQKuTNu7tgc2Dtw/cognito-confirmdevice-invalid-device-key-given-when-logging-in-with-user-srp-auth-mfa#ANA-ld3QusSh25uaYFZY468Q
    session.0 = user_id.to_string();

    let parameters = srp_client.verify(secret_block, user_id, salt, srp_b)?;

    let mut builder = cognito_client
        .respond_to_auth_challenge()
        .challenge_name(ChallengeNameType::PasswordVerifier)
        .set_session(session.1.clone())
        .client_id(CLIENT_ID)
        .challenge_responses("USERNAME", user_id)
        .challenge_responses(
            "PASSWORD_CLAIM_SECRET_BLOCK",
            parameters.password_claim_secret_block,
        )
        .challenge_responses(
            "PASSWORD_CLAIM_SIGNATURE",
            parameters.password_claim_signature,
        )
        .challenge_responses("TIMESTAMP", &parameters.timestamp);

    if let Some(trusted_device) = &user.device {
        builder = builder.challenge_responses("DEVICE_KEY", &trusted_device.device_key);
    }

    Ok(builder.send().await?)
}
