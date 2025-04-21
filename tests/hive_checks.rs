use dotenvy_macro::dotenv;
use hive_client::authentication::{TrustedDevice, User};
use hive_client::products::Product;
use hive_client::Client;

#[tokio::test]
pub async fn test_listing_operations() {
    let mut client = Client::new("Home Automation").await;

    let user = User::new(
        dotenv!("LIVE_USER_EMAIL"),
        dotenv!("LIVE_USER_PASSWORD"),
        Some(TrustedDevice::new(
            dotenv!("LIVE_TRUSTED_DEVICE_PASSWORD"),
            dotenv!("LIVE_TRUSTED_DEVICE_GROUP_KEY"),
            dotenv!("LIVE_TRUSTED_DEVICE_KEY"),
        )),
    );

    client
        .login(user)
        .await
        .expect("Logging in with Hive should succeed");

    let quick_actions = client
        .get_actions()
        .await
        .expect("Listing quick actions should succeed");
    let devices = client
        .get_devices()
        .await
        .expect("Listing devices should succeed");
    let products = client
        .get_products()
        .await
        .expect("Listing products should succeed");

    let _ = client
        .get_weather("SW1A 1AA")
        .await
        .expect("Getting weather should succeed");

    assert!(
        !quick_actions.is_empty(),
        "Quick actions should not be empty"
    );
    assert!(!devices.is_empty(), "Devices should not be empty");
    assert!(products.len() >= 2, "Products should not be empty");
    assert!(
        products.iter().any(|Product { data, .. }| matches!(
            data,
            hive_client::products::ProductData::Heating { .. }
        )),
        "Products should contain a heating product"
    );
    assert!(
        products.iter().any(|Product { data, .. }| matches!(
            data,
            hive_client::products::ProductData::HotWater { .. }
        )),
        "Products should contain a hot water product"
    );

    client.logout().await;
}
