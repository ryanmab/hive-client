/// The main Hive API URL - this is the URL for most of the API calls to make changes make
/// changes to Hive devices.
pub const BEEKEEPER_BASE_URL: &str = "https://beekeeper-uk.hivehome.com/1.0";

/// The URL for the weather API.
///
/// This is a separate API to the main Hive API and is used to get weather information.
pub const WEATHER_BASE_URL: &str = "https://weather.prod.bgchprod.info/weather";

pub enum Url<'a> {
    Products,
    Node {
        r#type: Option<&'a str>,
        id: Option<&'a str>,
    },
    Actions {
        id: Option<&'a str>,
        activate: bool,
    },
    Device,
    Weather,
}

pub fn get_base_url(url: &Url<'_>) -> String {
    match url {
        /*
         * Non-idempotent endpoints to set state
         */
        Url::Node {
            r#type: Some(r#type),
            id: Some(id),
        } => {
            format!("{}/{}/{}/{}", BEEKEEPER_BASE_URL, "nodes", r#type, id)
        }
        Url::Actions {
            id: Some(id),
            activate,
        } => match activate {
            true => format!("{}/{}/{}/quick-action", BEEKEEPER_BASE_URL, "actions", id),
            false => format!("{}/{}/{}", BEEKEEPER_BASE_URL, "actions", id),
        },
        Url::Actions { .. } => format!("{}/{}", BEEKEEPER_BASE_URL, "actions"),

        /*
         * Idempotent endpoints to list data
         */
        Url::Device => format!("{}/{}", BEEKEEPER_BASE_URL, "devices"),
        Url::Products => format!("{}/{}", BEEKEEPER_BASE_URL, "products"),
        Url::Node { .. } => {
            format!("{}/{}", BEEKEEPER_BASE_URL, "nodes")
        }

        /*
         * Weather endpoint
         */
        Url::Weather => WEATHER_BASE_URL.to_string(),
    }
}
