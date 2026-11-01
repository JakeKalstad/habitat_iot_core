use std::collections::HashMap;

use serde_derive::{Serialize, Deserialize};

pub const BRAND: &str = "habitat_iot";
pub const HUB_HOST: &str = "habitat_iot";
pub const HUB_MQTT_DNS: &str = "habitat_iot_hub_mqtt";
pub const HUB_DNS: &str = "habitat_iot_hub";

pub const SERVICE_NAME: &str = "_habitat_iot._tcp.local";
pub const SERVICE_TYPE: &str = "_habitat_iot";

pub const AP_PREFIX: &str = "HABITAT-";
// data bucket names
pub const TEMPERATURE_HUMIDITY: &str = "temp_humidity";
pub const TEMPERATURE_PRESSURE: &str = "temp_pressure";
pub const MOTION_SENSOR: &str = "motion_sensor";
pub const SERVO: &str = "servo";
pub const MOTOR: &str = "motor";
pub const DISPLAY: &str = "display";
pub const REED_SWITCH: &str = "reed_switch";
pub const AIR_QUALITY: &str = "air_quality";
pub const WATER_SENSOR: &str = "water_sensor";
pub const BARE: &str = "bare";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Heartbeat {
    pub version: String,
    pub timestamp: u128,
    pub model: String,
    pub mqtt_enabled: bool,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub connection_type: String,
    pub hardware_compatible: Vec<String>,
    #[serde(default)]
    pub pins: HashMap<String, i32>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Device {
    pub id: String,
    pub version: String,
    pub model: String,
    pub features: HashMap<String, Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AirQualityResult {
    pub co2_ppm: u16,
    pub tvoc_ppb: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceData {
    pub data: HashMap<String, String>,
    pub timestamp: i128,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemperaturePressureResult {
    pub temperature: f64,
    pub pressure: f64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemperatureHumidityResult {
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BareResult {
    pub message: String,
} 

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoolResult {
    pub state: bool,
} 

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleResult {
    pub value: u16,
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
