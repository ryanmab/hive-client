use crate::authentication::user::AccountDevice;
use crate::authentication::{LoginSession, User};
use crate::constants::CLIENT_ID;
use crate::AuthenticationError;
use aws_cognito_srp::SrpClient;
use aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput;
use aws_sdk_cognitoidentityprovider::types::ChallengeNameType;

pub async fn handle_challenge(
    user: &User,
    cognito_client: &aws_sdk_cognitoidentityprovider::Client,
    srp_client: &SrpClient<aws_cognito_srp::TrackedDevice>,
    session: &LoginSession,
) -> Result<RespondToAuthChallengeOutput, AuthenticationError> {
    let parameters = srp_client.get_auth_parameters();

    let mut builder = cognito_client
        .respond_to_auth_challenge()
        .challenge_responses("SRP_A", parameters.a)
        .challenge_responses("USERNAME", session.0.clone())
        .set_session(session.1.clone())
        .client_id(CLIENT_ID)
        .challenge_name(ChallengeNameType::DeviceSrpAuth);

    if let Some(AccountDevice::Trusted(trusted_device)) = &user.account_device {
        builder = builder.challenge_responses("DEVICE_KEY", &trusted_device.device_key);
    }

    Ok(builder.send().await?)
}
