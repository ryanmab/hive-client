use crate::RefreshError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
#[non_exhaustive]
/// Errors that can occur while trying to communicate with the Hive API.
pub enum ApiError {
    #[error("An error occurred with the request sent to the Hive API: {0}")]
    /// The request to the Hive API failed to return a successful response.
    RequestError(#[from] reqwest::Error),

    #[error("An error occurred while decoding the response from the Hive API: {0}")]
    /// The response from the Hive API was valid, but could not be decoded.
    InvalidResponse(#[from] serde_json::Error),

    #[error("An error occurred while trying to refresh the authentication tokens")]
    /// When refreshing the authentication tokens an error occurred.
    RefreshError(#[from] RefreshError),
}
