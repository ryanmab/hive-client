use crate::client::authentication::{HiveAuth, Tokens};
use crate::AuthenticationError;

impl HiveAuth {
    pub async fn logout(&self, tokens: &Tokens) -> Result<(), AuthenticationError> {
        log::info!("Beginning to invalidate tokens with Cognito.");

        self.cognito
            .global_sign_out()
            .access_token(tokens.access_token.clone())
            .send()
            .await?;

        Ok(())
    }
}
