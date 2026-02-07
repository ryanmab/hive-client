use crate::AuthenticationError;
use crate::authentication::LoginSession;
use crate::constants::CLIENT_ID;
use aws_cognito_srp::SrpClient;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;

pub async fn handle_challenge(
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    device_srp_client: Option<&SrpClient<aws_cognito_srp::TrackedDevice>>,
    session: &LoginSession,
    code: &str,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let mut builder = cognito_client
        .respond_to_auth_challenge()
        .challenge_responses("SMS_MFA_CODE", code)
        .challenge_responses("USERNAME", session.0.clone())
        .set_session(Option::clone(&session.1))
        .client_id(CLIENT_ID)
        .challenge_name(ChallengeNameType::SmsMfa);

    if let Some(device_key) = device_srp_client
        .map(|device_srp_client| device_srp_client.get_auth_parameters().device_key)
    {
        builder = builder.challenge_responses("DEVICE_KEY", device_key);
    }

    Ok(builder.send().await?)
}
