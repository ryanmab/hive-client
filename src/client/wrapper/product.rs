use crate::products::{Product, States};
use crate::{ApiError, Client};

impl Client {
    /// Get all of the Hive products setup in the Hive account.
    ///
    /// For example, the Heating or Hot Water products.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hive_client::authentication::{TrustedDevice, User};
    /// use hive_client::products::{Product, ProductData, State, States};
    ///
    /// # tokio_test::block_on(async {
    /// let client = hive_client::Client::new("Home Automation");
    ///
    /// let trusted_device = Some(TrustedDevice::new(
    ///     "device_password",
    ///     "device_group_key",
    ///     "device_key"
    /// ));
    ///
    /// let attempt = client.login(User::new("example@example.com", "example"), trusted_device).await;
    ///
    /// if let Ok(_) = attempt {
    ///     // Login was successful
    ///
    ///     let products = client.get_products()
    ///         .await
    ///         .expect("Products should be retrieved");
    ///     
    ///     println!("{:?}", products);
    /// }
    /// # })
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the list of products could not be retrieved.
    pub async fn get_products(&self) -> Result<Vec<Product<'_>>, ApiError> {
        self.api
            .get_product_data(&*self.refresh_tokens_if_needed().await?)
            .await
            .map(|products| {
                products
                    .into_iter()
                    .map(|data| Product::new(self, data))
                    .collect()
            })
    }

    /// Set a series of states on a product by a given ID.
    ///
    /// Wrapped by [`Product::set_state`] to set the states on a returned Product.
    pub(crate) async fn set_product_state(
        &self,
        product_id: &str,
        r#type: &str,
        states: States,
    ) -> Result<bool, ApiError> {
        self.api
            .set_product_state(
                &*self.refresh_tokens_if_needed().await?,
                product_id,
                r#type,
                states,
            )
            .await
    }
}
