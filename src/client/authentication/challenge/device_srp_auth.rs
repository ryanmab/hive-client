use crate::AuthenticationError;
use crate::authentication::LoginSession;
use crate::constants::CLIENT_ID;
use aws_cognito_srp::SrpClient;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;

pub async fn handle_challenge(
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    device_srp_client: &SrpClient<aws_cognito_srp::TrackedDevice>,
    session: &LoginSession,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let aws_cognito_srp::DeviceAuthenticationParameters { a, device_key, .. } =
        device_srp_client.get_auth_parameters();

    Ok(cognito_client
        .respond_to_auth_challenge()
        .challenge_responses("SRP_A", a)
        .challenge_responses("USERNAME", session.0.clone())
        .set_session(session.1.clone())
        .client_id(CLIENT_ID)
        .challenge_name(ChallengeNameType::DeviceSrpAuth)
        .challenge_responses("DEVICE_KEY", device_key)
        .send()
        .await?)
}
