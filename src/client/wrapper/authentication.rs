use crate::authentication::{AccountDevice, ChallengeResponse, Tokens, TrustedDevice, User};
use crate::{ApiError, AuthenticationError, Client};
use chrono::Utc;
use std::sync::Arc;

impl Client {
    /// Login to Hive as a User.
    ///
    /// This user may _optionally_ have a trusted device associated with their account ([`Client::confirm_device`])
    /// which can be provided as [`TrustedDevice`]. If provided, this allows for a simpler login process that does not
    /// require manual Two Factor Authentication ([`ChallengeResponse::SmsMfa`]).
    ///
    /// # Examples
    ///
    /// ## Login _with_ a trusted device
    ///
    /// If the user has previously logged in and set the Client as a trusted device
    /// ([`Client::confirm_device`]), the trusted device can be provided to skip some
    /// authentication challenges.
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new().await;
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
    /// let mut client = hive_client::Client::new().await;
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", None)).await;
    ///
    /// match attempt {
    ///     Ok(_) => {
    ///        // Login was successful
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
    /// call [`Client::respond_to_challenge`] to continue with the login process.
    pub async fn login(&self, user: User) -> Result<(), AuthenticationError> {
        let mut u = self.user.lock().await;
        let user = u.insert(user);

        let (tokens, untrusted_device) = self.auth.login(user).await?;

        if let Some(untrusted_device) = untrusted_device {
            user.account_device
                .replace(AccountDevice::Untrusted(untrusted_device));
        }

        self.tokens.lock().await.replace(Arc::new(tokens));

        Ok(())
    }

    /// Respond to a challenge issued by Hive during the login process.
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
    /// let mut client = hive_client::Client::new().await;
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", None)).await;
    ///
    /// match attempt {
    ///     Ok(_) => {
    ///        // Login was successful
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
    /// Returns an error if the challenge submission was unsuccessful. If this
    /// happens, the caller must check the error type and handle it accordingly.
    pub async fn respond_to_challenge(
        &mut self,
        challenge_response: ChallengeResponse,
    ) -> Result<(), AuthenticationError> {
        let mut user = self.user.lock().await;

        let (tokens, untrusted_device) = self
            .auth
            .respond_to_challenge(
                user.as_ref().ok_or(AuthenticationError::NotLoggedIn)?,
                challenge_response,
            )
            .await?;

        if let Some(untrusted_device) = untrusted_device {
            user.as_mut()
                .ok_or(AuthenticationError::NotLoggedIn)?
                .account_device
                .replace(AccountDevice::Untrusted(untrusted_device));
        }

        drop(user);

        self.tokens.lock().await.replace(Arc::new(tokens));

        Ok(())
    }

    /// Logout from Hive.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    ///
    /// # tokio_test::block_on(async {
    /// let mut client = hive_client::Client::new().await;
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
    /// client.logout()
    ///     .await
    ///     .expect("Logout should succeed");
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the logout process fails.
    pub async fn logout(&mut self) -> Result<(), AuthenticationError> {
        if self.tokens.lock().await.as_ref().is_none() {
            return Ok(());
        }

        self.user.lock().await.take();

        let tokens = self.tokens.lock().await.take();

        if let Some(tokens) = tokens {
            self.auth.logout(&tokens).await?;
        }

        log::info!("Taken API client and user details now that logout is complete.");

        Ok(())
    }

    /// Set the currently logged in [`Client`], as a trusted device against the User.
    ///
    /// The returned [`TrustedDevice`] can then be used during subsequent login attempts to
    /// skip additional challenges (such as [`ChallengeResponse::SmsMfa`], and simplify the
    /// login process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{ChallengeResponse, TrustedDevice, User};
    /// use hive_client::AuthenticationError;
    ///
    /// # tokio_test::block_on(async {
    /// let mut client = hive_client::Client::new().await;
    ///
    /// let attempt = client.login(User::new("example@example.com", "example", None)).await;
    ///
    /// match attempt {
    ///     Ok(_) => {
    ///        // Login was successful
    ///     },
    ///     Err(AuthenticationError::NextChallenge(challenge)) => {
    ///         // Hive prompted for a challenge to be responded to before
    ///         // authentication can be completed.
    ///
    ///         // Handle the challenge accordingly, and respond to the challenge.
    ///         let sms_code = "123456";
    ///         let response = client.respond_to_challenge(ChallengeResponse::SmsMfa(sms_code.to_string()))
    ///             .await
    ///             .expect("Expected challenge response to succeed.");
    ///     },
    ///     Err(_) => {
    ///       // Login failed, respond accordingly.
    ///     }
    /// }
    ///
    ///  let trusted_device = client.confirm_device("hive-client").await;
    ///
    ///  assert!(&trusted_device.is_ok());
    ///
    ///  // The trusted device can now be used to skip the challenge in the future.
    ///  println!("{:?}", trusted_device);
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the device confirmation process fails.
    pub async fn confirm_device(
        &self,
        device_name: &str,
    ) -> Result<TrustedDevice, AuthenticationError> {
        let mut user = self.user.lock().await;
        let user = user.as_mut().ok_or(AuthenticationError::NotLoggedIn)?;

        let device = user
            .account_device
            .take_if(|device| matches!(device, AccountDevice::Untrusted(_)))
            .ok_or(AuthenticationError::NotLoggedIn)?;

        if let AccountDevice::Untrusted(untrusted_device) = device {
            return self
                .auth
                .confirm_device(
                    device_name,
                    &user.username,
                    untrusted_device,
                    &*self
                        .refresh_tokens_if_needed()
                        .await
                        .map_err(|_| AuthenticationError::AuthenticationRefreshFailed)?,
                )
                .await;
        }

        // Device is already trusted, no need to confirm again.
        Err(AuthenticationError::DeviceAlreadyTrusted)
    }

    /// Refresh the currently stored [`Tokens`], if they have expired.
    ///
    /// This is commonly used by wrapper API methods, before performing a call to
    /// the Hive API, to ensure their tokens are fresh and ready to be used.
    pub(crate) async fn refresh_tokens_if_needed(&self) -> Result<Arc<Tokens>, ApiError> {
        let mut token_to_refresh = self.tokens.lock().await;

        match token_to_refresh.as_ref() {
            mut current_tokens
                if current_tokens.is_some_and(|tokens| tokens.expires_at < Utc::now()) =>
            {
                let current_tokens = current_tokens
                    .take()
                    .expect("Tokens must already be present to need to refresh");

                let replacement_tokens = Arc::new(
                    self.auth
                        .refresh_tokens(Arc::clone(current_tokens))
                        .await
                        .map_err(|_| ApiError::AuthenticationRefreshFailed)?,
                );

                token_to_refresh.replace(Arc::clone(&replacement_tokens));

                drop(token_to_refresh);

                log::info!(
                    "Tokens have been refreshed successfully. New expiration time: {}",
                    replacement_tokens.expires_at,
                );

                Ok(Arc::clone(&replacement_tokens))
            }
            Some(current_tokens) => Ok(Arc::clone(current_tokens)),
            None => Err(ApiError::AuthenticationRefreshFailed),
        }
    }
}
