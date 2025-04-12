#[cfg(test)]
use dotenvy_macro::dotenv;

/// The ID of the client used to connect to Hive's Cognito User Pool.
///
/// This is the web portal client ID.
///
/// This can be found in the source code of the [Hive web portal](https://sso.hivehome.com/)
/// in the property `window.HiveSSOCognitoClientId`.
#[cfg(not(test))]
pub const CLIENT_ID: &str = "3rl4i0ajrmtdm8sbre54p9dvd9";
#[cfg(test)]
pub const CLIENT_ID: &str = dotenv!("CLIENT_ID");

/// The ID of the Cognito User Pool.
///
/// This can be found in the source code of the [Hive web portal](https://sso.hivehome.com/)
/// in the property `window.HiveSSOPoolId`.
#[cfg(not(test))]
pub const POOL_ID: &str = "eu-west-1_SamNfoWtf";
#[cfg(test)]
pub const POOL_ID: &str = dotenv!("POOL_ID");

/// The region the user pool is in.
///
/// This is available by looking at the start of [`POOL_ID`].
#[cfg(not(test))]
pub const REGION: &str = "eu-west-1";
#[cfg(test)]
pub const REGION: &str = dotenv!("REGION");
