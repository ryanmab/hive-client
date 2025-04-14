use crate::client::authentication::ChallengeRequest;
use aws_cognito_srp::SrpError;
use aws_sdk_cognitoidentityprovider::error::SdkError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
#[non_exhaustive]
/// Errors that can occur while trying to communicate with the Hive Authentication servers.
pub enum AuthenticationError {
    #[error("The session token was not found in the response")]
    /// No Access Token was issued in the response from the Hive authentication servers.
    AccessTokenNotValid,

    #[error("The presented challenge is not supported. Challenge was: {0}")]
    /// The challenge presented by the Hive authentication servers is not supported by this crate.
    UnsupportedChallenge(String),

    #[error(transparent)]
    /// The request to begin the authentication flow failed.
    LoginFailed(
        #[from]
        SdkError<aws_sdk_cognitoidentityprovider::operation::initiate_auth::InitiateAuthError>,
    ),

    #[error(transparent)]
    /// The request to respond to a challenge during the authentication flow failed.
    ChallengeFailed(
        #[from]
        SdkError<aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeError>,
    ),

    #[error(transparent)]
    /// The request to logout the user failed.
    LogoutFailed(
        #[from]
        SdkError<aws_sdk_cognitoidentityprovider::operation::global_sign_out::GlobalSignOutError>,
    ),

    #[error("The challenge was not handled correctly")]
    /// A parameter which was expected to be present in the challenge was not found.
    MissingChallengeParameter(String),

    #[error("An error occurred while trying to authenticate the user")]
    /// An error occured while trying to complete the [Secure Remote Password (SRP)](https://github.com/ryanmab/aws-cognito-srp) authentication challenges.
    SrpFailed(
        #[from]
        SrpError,
    ),

    #[error("A challenge was requested")]
    /// A challenge was requested by the Hive authentication servers which requires manual intervention.
    ///
    /// For example, a SMS MFA code was sent to the user's phone number.
    NextChallenge(ChallengeRequest),

    /// The request to confirm the device (to make it a [`crate::authentication::TrustedDevice`]) failed.
    DeviceConfirmationError(
        #[from]
        DeviceConfirmationError
    ),

    #[error("The API call failed as the authentication could not be refreshed")]
    /// The request to refresh the authentication tokens failed.
    AuthenticationRefreshFailed,

    #[error("There is currently no valid authentication in progress")]
    /// The authentication flow is not currently in progress, and the user is not logged in.
    NoAuthenticationInProgress,

    #[error("Unable to continue with the authentication flow as the user is not logged in")]
    /// No authentication flow has been started.
    NotLoggedIn,

    #[error("The device has already been confirmed")]
    /// The device being confirmed is already trusted, meaning no confirmation is needed.
    DeviceAlreadyTrusted,
}

#[derive(Error, Debug)]
#[error(transparent)]
/// Errors that can occur while trying to confirm a device in order to
/// make it a [`crate::authentication::TrustedDevice`].
pub enum DeviceConfirmationError {
    #[error(transparent)]
    /// The request to confirm the device failed.
    ConfirmationFailed(
        #[from]
        SdkError<aws_sdk_cognitoidentityprovider::operation::confirm_device::ConfirmDeviceError>,
    ),

    #[error(transparent)]
    /// The request to update the device status failed.
    StatusUpdateFailed(
        #[from]
        SdkError<aws_sdk_cognitoidentityprovider::operation::update_device_status::UpdateDeviceStatusError>,
    ),

    #[error("The device being confirmed is already tracked")]
    /// The device being confirmed is already tracked, meaning no confirmation is needed.
    DeviceAlreadyTracked,
}
