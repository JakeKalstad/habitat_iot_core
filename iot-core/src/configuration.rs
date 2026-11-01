use serde::Deserialize;

pub struct UserConfig {
    pub mqtt_url: String,
    pub mqtt_user: String,
    pub mqtt_password: String,
    pub mqtt_enabled: bool,

    pub device_name: String,
    pub device_location: String,

    pub wifi_ssid: String,
    pub wifi_key: String,
}