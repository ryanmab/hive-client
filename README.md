[![Coverage](https://api.coveragerobot.com/v1/graph/github/ryanmab/hive-client/badge.svg?token=39c3c1957ea32896db7e1cc0d2f5c8450b6d193f0ddac0d104)](https://coveragerobot.com)
[![Crates.io Version](https://img.shields.io/crates/v/hive-client)](https://crates.io/crates/hive-client)
![Crates.io Total Downloads](https://img.shields.io/crates/d/hive-client)
[![docs.rs](https://img.shields.io/docsrs/hive-client)](https://docs.rs/hive-client)
[![Build](https://github.com/ryanmab/hive-client/actions/workflows/build.yml/badge.svg)](https://github.com/ryanmab/hive-client/actions/workflows/build.yml)
![GitHub License](https://img.shields.io/github/license/ryanmab/hive-client)

<!-- cargo-rdme start -->

# Hive Client

The Hive Client crate provides a client for interfacing with [Hive](https://www.hivehome.com/) smart home systems.

## Usage
```toml
[dependencies]
hive-client = "0.0.5"
```

## Capabilities

1. Authenticate with Hive.
2. Setup trusted devices in a Hive account.
3. Request live Weather information directly from Hive.
4. List [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) configured in a Hive account.
5. Activate [Quick Actions](https://www.hivehome.com/ie/support/Help_Using_Hive/HUH_General/What-are-Quick-Actions) on demand.
6. List Devices associated with a Hive account (Hubs, Thermostats, Hubs, etc).
7. List Products currently running in a Hive account.
8. Change state of Products in a Hive account (Boost heating/hot water, change target
   temperature, etc).

## Examples

More in-depth examples can be found in documentation comments on the Client methods.

### Trigger a Quick Action

```rust
use hive_client::authentication::{TrustedDevice, User};

let client = hive_client::Client::new("Home Automation");

let trusted_device = Some(TrustedDevice::new(
    "device_password",
    "device_group_key",
    "device_key"
));

let attempt = client.login(User::new("example@example.com", "example"), trusted_device).await;

if let Ok(_) = attempt {
    // Login was successful

    let mut actions = client.get_actions()
        .await
        .expect("Quick action should be retrieved");

    if let Some(mut first_action) = actions.first_mut() {
        let was_activated = first_action.activate()
            .await
            .expect("Quick action should be activated");
    }
}
```

### Set Target Temperature of Heating

```rust
use hive_client::authentication::{TrustedDevice, User};
use hive_client::products::{Product, ProductData, State, States};

let client = hive_client::Client::new("Home Automation");

let trusted_device = Some(TrustedDevice::new(
    "device_password",
    "device_group_key",
    "device_key"
));

let attempt = client.login(User::new("example@example.com", "example"), trusted_device).await;

if let Ok(_) = attempt {
    // Login was successful

    let products = client.get_products()
        .await
        .expect("Products should be retrieved");

    if let Some(mut heating) = products.into_iter().find(|Product { data, .. }| matches!(data, ProductData::Heating { .. })) {
        let was_set = heating.set_state(States(vec!(State::TargetTemperature(18.0))))
            .await
            .expect("Product state should be set");
    }
}
```

### Retrieve Current Weather

```rust
use hive_client::authentication::{TrustedDevice, User};
use hive_client::products::{Product, ProductData, State, States};

let client = hive_client::Client::new("Home Automation");

let trusted_device = Some(TrustedDevice::new(
    "device_password",
    "device_group_key",
    "device_key"
));

let attempt = client.login(User::new("example@example.com", "example"), trusted_device).await;

if let Ok(_) = attempt {
    // Login was successful

    let postcode = "SW1A 1AA";

    let weather = client.get_weather(postcode)
        .await
        .expect("Weather should be retrieved");

    // Example: It is currently 18.0C with clear skies at Buckingham Palace
    println!("It is currently {} with {} at Buckingham Palace", weather.data.temperature, weather.data.description);
}
```

## Contributing

There are _tons_ of features which could be added to the crate. If you'd like to contribute, please
feel free to open an issue or a pull request.

Examples of features which could be added:
1. Better parity between the Hive API and the structs.
2. Support for controlling Holiday Mode.
3. Support for modifying the schedule of a Hive Device.
4. Support for other Hive products (e.g. Hive Lights, Smart Plugs, Motion Sensors, etc).

### Testing
Many of the tests require that an AWS Cognito User Pool, configured with SRP authentication and device
tracking, be available to act as a mock server.

The setup of a suitable User Pool is fully automated with Terraform (see the [`infrastructure`](infrastructure/) folder).

#### Set up

In order to setup the test environment, Terraform needs to be installed and AWS credentials need to be
configured locally.

Once this is done, running `apply` should setup the Terraform backend, and the user pool and app client in the correct state:
```sh
# Setup state backend
cd infrastructure/state && terraform init && terraform apply

# Setup user pool
cd infrastructure/tests && terraform init --backend-config="./local.config" && terraform apply
```

After the user pool is set up, multiple environment variables need to be set in a `.env` file.

The `.env` file can be created by using `.env.example` as a template:
```sh
cp .env.example .env
```

#### Running tests

The tests can be run with:
```sh
cargo test
```

#### Tear down

The test environment can be torn down at any point with:
```sh
cd infrastructure/tests && terraform destroy
```

<!-- cargo-rdme end -->
