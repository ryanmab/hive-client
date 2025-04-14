use crate::authentication::{ChallengeResponse, Tokens, TrustedDevice, User};
use crate::{ApiError, AuthenticationError, Client};
use chrono::Utc;
use std::sync::Arc;

impl Client {
    /// Login to Hive as a User.
    ///
    /// This user may _optionally_ have a trusted device associated with their account.
    ///
    /// If provided, this induces a simpler login flow, which does not require Two Factor
    /// Authentication ([`ChallengeResponse::SmsMfa`]).
    ///
    /// If not provided, a new device will be automatically confirmed with Hive during the login flow.
    ///
    /// # Examples
    ///
    /// ## Login _with_ a trusted device
    ///
    /// If the user has previously logged in and set the Client as a trusted device , the trusted
    /// device can be provided to skip some authentication challenges.
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new("Home Automation").await;
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", trusted_device)).await;
    ///
    /// // Login shouldn't require any additional challenges, as a remembered device was provided.
    /// assert!(attempt.is_ok());
    /// # })
    /// ```
    ///
    /// ## Login _without_ a trusted device
    ///
    /// ```no_run
    /// use hive_client::authentication::{ChallengeResponse, TrustedDevice, User};
    /// use hive_client::AuthenticationError;
    ///
    /// # tokio_test::block_on(async {
    /// let mut client = hive_client::Client::new("Home Automation").await;
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", None)).await;
    ///
    /// match attempt {
    ///     Ok(trusted_device) => {
    ///        // Login was successful.
    ///        //
    ///        // If a trusted device has been returned this can be used to authenticate in the future.
    ///     },
    ///     Err(AuthenticationError::NextChallenge(challenge)) => {
    ///        // Hive prompted for a challenge to be responded to before
    ///        // authentication can be completed.
    ///
    ///        // Handle the challenge accordingly, and respond to the challenge.
    ///        let sms_code = "123456";
    ///        let response = client.respond_to_challenge(ChallengeResponse::SmsMfa(sms_code.to_string())).await;
    ///
    ///        assert!(response.is_ok());
    ///     },
    ///     Err(_) => {
    ///       // Login failed, respond accordingly.
    ///     }
    /// }
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if Hive did not immediately return an active
    /// session.
    ///
    /// This can happen if the credentials are invalid, or if Hive prompt for
    /// a challenge in order to process ([`AuthenticationError::NextChallenge`]).
    ///
    /// In the latter case, the caller must generate a [`ChallengeResponse`] and
    /// call [`Client::respond_to_challenge`] to continue with the authentication process.
    pub async fn login(
        &self,
        user: User,
    ) -> Result<Option<Arc<TrustedDevice>>, AuthenticationError> {
        let mut u = self.user.lock().await;
        let user = u.insert(user);

        let (tokens, untrusted_device) = self.auth.login(user).await?;

        if let Some(untrusted_device) = untrusted_device {
            user.device.replace(Arc::new(
                self.auth
                    .confirm_device(
                        &user.username,
                        &self.friendly_name,
                        untrusted_device,
                        &tokens,
                    )
                    .await?,
            ));
        }

        self.tokens.lock().await.replace(Arc::new(tokens));

        Ok(user.device.as_ref().map(Arc::clone))
    }

    /// Respond to a challenge issued by Hive during the authentication process.
    ///
    /// This is typically used to handle Two Factor Authentication (2FA) challenges, but could be any
    /// challenge issued by Hive that requires a response from the user ([`Client::login`])
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{ChallengeResponse, TrustedDevice, User};
    /// use hive_client::AuthenticationError;
    ///
    /// # tokio_test::block_on(async {
    /// let mut client = hive_client::Client::new("Home Automation").await;
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", None)).await;
    ///
    /// match attempt {
    ///     Ok(trusted_device) => {
    ///         // Login was successful.
    ///         //
    ///         // If a trusted device has been returned this can be used to authenticate in the future.
    ///     },
    ///     Err(AuthenticationError::NextChallenge(challenge)) => {
    ///         // Hive prompted for a challenge to be responded to before
    ///         // authentication can be completed.
    ///
    ///         // Handle the challenge accordingly, and respond to the challenge.
    ///         let sms_code = "123456";
    ///         let response = client.respond_to_challenge(ChallengeResponse::SmsMfa(sms_code.to_string())).await;
    ///
    ///         if let Ok(trusted_device) = response {
    ///             // Login was successful.
    ///             //
    ///             // If a trusted device has been returned this can be used to authenticate in the future.
    ///         } else {
    ///             // Challenge failed, respond accordingly.
    ///         }
    ///     },
    ///     Err(_) => {
    ///         // Login failed, respond accordingly.
    ///     }
    /// }
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the challenge submission was unsuccessful. If this
    /// happens, the caller must check the error type and handle it accordingly.
    pub async fn respond_to_challenge(
        &mut self,
        challenge_response: ChallengeResponse,
    ) -> Result<Option<Arc<TrustedDevice>>, AuthenticationError> {
        let mut user = self.user.lock().await;
        let user = user.as_mut().ok_or(AuthenticationError::NotLoggedIn)?;

        let (tokens, untrusted_device) = self
            .auth
            .respond_to_challenge(user, challenge_response)
            .await?;

        if let Some(untrusted_device) = untrusted_device {
            user.device.replace(Arc::new(
                self.auth
                    .confirm_device(
                        &user.username,
                        &self.friendly_name,
                        untrusted_device,
                        &tokens,
                    )
                    .await?,
            ));
        }

        self.tokens.lock().await.replace(Arc::new(tokens));

        Ok(user.device.as_ref().map(Arc::clone))
    }

    /// Logout from Hive.
    ///
    /// Note: This only clears the client, it does not perform any operations on the Hive Account.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    ///
    /// # tokio_test::block_on(async {
    /// let mut client = hive_client::Client::new("Home Automation").await;
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", trusted_device)).await;
    ///
    /// // Login shouldn't require any additional challenges, as a remembered device was provided.
    /// assert!(attempt.is_ok());
    ///
    /// client.logout().await;
    /// # })
    /// ```
    pub async fn logout(&mut self) {
        // Note that we're not calling any operations in Cognito here. Instead,
        // we're just dropping the tokens and user from the Client.
        //
        // There are a number of options for invalidating refresh tokens tokens,
        // however the one we want is the Revoke Operation API call, which is not
        // enabled in Hive's user pool.
        //
        // It's possible to use the Global Sign out endpoint, but this would sign out
        // everyone using the same user account, which is not ideal.
        //
        // https://docs.aws.amazon.com/cognito/latest/developerguide/token-revocation.html
        drop(self.user.lock().await.take());
        drop(self.tokens.lock().await.take());

        log::info!("Logout is complete, tokens have been dropped.");
    }

    /// Refresh the currently stored [`Tokens`], if they have expired.
    ///
    /// This is commonly used by wrapper API methods, before performing a call to
    /// the Hive API, to ensure their tokens are fresh and ready to be used.
    pub(crate) async fn refresh_tokens_if_needed(&self) -> Result<Arc<Tokens>, ApiError> {
        let mut token_to_refresh = self.tokens.lock().await;

        match token_to_refresh.as_ref() {
            mut current_tokens
                if current_tokens.is_some_and(|tokens| tokens.expires_at <= Utc::now()) =>
            {
                let current_tokens = current_tokens
                    .take()
                    .expect("Tokens must already be present to need to refresh");

                let replacement_tokens = {
                    let user = self.user.lock().await;

                    Arc::new(
                        self.auth
                            .refresh_tokens(
                                user.as_ref().and_then(|user| {
                                    user.device
                                        .as_ref()
                                        .map(|device| device.device_key.as_str())
                                }),
                                Arc::clone(current_tokens),
                            )
                            .await
                            .map_err(ApiError::AuthenticationRefreshFailed)?,
                    )
                };

                token_to_refresh.replace(Arc::clone(&replacement_tokens));

                drop(token_to_refresh);

                log::info!(
                    "Tokens have been refreshed successfully. New expiration time: {}",
                    replacement_tokens.expires_at,
                );

                Ok(Arc::clone(&replacement_tokens))
            }
            Some(current_tokens) => Ok(Arc::clone(current_tokens)),
            None => Err(ApiError::AuthenticationRefreshFailed(
                AuthenticationError::NotLoggedIn,
            )),
        }
    }
}
