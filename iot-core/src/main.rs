use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{format, Write};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use esp_idf_hal::adc::oneshot::AdcDriver;

use esp_idf_hal::gpio::{self, ADCPin, AnyIOPin, IOPin, InputPin, OutputPin};
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::{self, Dma, SpiConfig, SpiDeviceDriver, SpiDriver, SPI2};
use esp_idf_hal::{delay, i2c};
use esp_idf_svc::eventloop::EspSystemEventLoop;
// use esp_idf_svc::hal::adc::{attenuation, AdcChannelDriver, AdcDriver};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::wifi::*;

#[cfg(any(feature = "air_quality", feature = "temp_pressure"))]
use esp_idf_svc::hal::i2c::I2cConfig;

use esp_idf_svc::hal::gpio::PinDriver;

#[cfg(feature = "servo")]
use servo::{Servo, ServoConfig};

#[cfg(feature = "air_quality")]
use sgp30::Sgp30;

#[cfg(feature = "motor")]
pub mod l298n;

use crate::configuration::UserConfig;

#[cfg(feature = "display")]
use pcd8544;
// pub mod PCD8544;

pub mod ap_routes;
pub mod common;
pub mod configuration;
pub mod connected_routes;
pub mod data;
pub mod mdns;
pub mod networking;
pub mod servo;
use esp_idf_hal::gpio::{
    Gpio0, Gpio1, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio17, Gpio18, Gpio19,
    Gpio2, Gpio21, Gpio22, Gpio23, Gpio25, Gpio26, Gpio27, Gpio3, Gpio32, Gpio33, Gpio34, Gpio35,
    Gpio36, Gpio39, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9, Pins,
};

static CONFIG_JSON: &str = include_str!("config.json");
static WIFI_NAMESPACE: &str = "wifi_ns";

static TAG_SSID_STR: &str = "ssid";
static TAG_KEY_STR: &str = "wifi_key";

static DEVICE_NAME_STR: &str = "device_name";
static DEVICE_LOCATION_STR: &str = "device_location";

static MQTT_URL_STR: &str = "mqtt_url";
static MQTT_PASSWORD: &str = "mqtt_password";
static MQTT_USER: &str = "mqtt_user";

static MQTT_ENABLED_STR: &str = "mqtt_enable";

static PRODUCT_CONFIGURATION_JSON: &str = include_str!("config.json");
enum GpioPin {
    Gpio0(Arc<Mutex<Gpio0>>),
    Gpio1(Arc<Mutex<Gpio1>>),
    Gpio2(Arc<Mutex<Gpio2>>),
    Gpio3(Arc<Mutex<Gpio3>>),
    Gpio4(Arc<Mutex<Gpio4>>),
    Gpio5(Arc<Mutex<Gpio5>>),
    Gpio6(Arc<Mutex<Gpio6>>),
    Gpio7(Arc<Mutex<Gpio7>>),
    Gpio8(Arc<Mutex<Gpio8>>),
    Gpio9(Arc<Mutex<Gpio9>>),
    Gpio10(Arc<Mutex<Gpio10>>),
    Gpio11(Arc<Mutex<Gpio11>>),
    Gpio12(Arc<Mutex<Gpio12>>),
    Gpio13(Arc<Mutex<Gpio13>>),
    Gpio14(Arc<Mutex<Gpio14>>),
    Gpio15(Arc<Mutex<Gpio15>>),
    Gpio16(Arc<Mutex<Gpio16>>),
    Gpio17(Arc<Mutex<Gpio17>>),
    Gpio18(Arc<Mutex<Gpio18>>),
    Gpio19(Arc<Mutex<Gpio19>>),
    Gpio21(Arc<Mutex<Gpio21>>),
    Gpio22(Arc<Mutex<Gpio22>>),
    Gpio23(Arc<Mutex<Gpio23>>),
    Gpio25(Arc<Mutex<Gpio25>>),
    Gpio26(Arc<Mutex<Gpio26>>),
    Gpio27(Arc<Mutex<Gpio27>>),
    Gpio32(Arc<Mutex<Gpio32>>),
    Gpio33(Arc<Mutex<Gpio33>>),
    Gpio34(Arc<Mutex<Gpio34>>),
    Gpio35(Arc<Mutex<Gpio35>>),
    Gpio36(Arc<Mutex<Gpio36>>),
    Gpio39(Arc<Mutex<Gpio39>>),
}
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let device_config: iot_common::Device =
        serde_json::from_str(PRODUCT_CONFIGURATION_JSON).expect("Read Configuration");

    let ssid_found = Arc::new(Mutex::new(false));
    let ssid = Arc::new(Mutex::new(String::new()));
    let password = Arc::new(Mutex::new(String::new()));
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().expect("Take event");
    let nvs = EspDefaultNvsPartition::take().expect("NVS Initialized");
    let pins = peripherals.pins;

    let mut gpio_map: HashMap<i32, Arc<Mutex<AnyIOPin>>> = HashMap::new();

    gpio_map.insert(0, Arc::new(Mutex::new(pins.gpio0.into())));
    gpio_map.insert(1, Arc::new(Mutex::new(pins.gpio1.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(2, Arc::new(Mutex::new(pins.gpio2.into())));
    gpio_map.insert(3, Arc::new(Mutex::new(pins.gpio3.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(4, Arc::new(Mutex::new(pins.gpio4.into())));
    gpio_map.insert(5, Arc::new(Mutex::new(pins.gpio5.into())));
    gpio_map.insert(6, Arc::new(Mutex::new(pins.gpio6.into())));
    gpio_map.insert(7, Arc::new(Mutex::new(pins.gpio7.into())));
    gpio_map.insert(8, Arc::new(Mutex::new(pins.gpio8.into())));
    gpio_map.insert(9, Arc::new(Mutex::new(pins.gpio9.into())));
    gpio_map.insert(10, Arc::new(Mutex::new(pins.gpio10.into())));
    gpio_map.insert(11, Arc::new(Mutex::new(pins.gpio11.into())));
    gpio_map.insert(12, Arc::new(Mutex::new(pins.gpio12.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(13, Arc::new(Mutex::new(pins.gpio13.into())));
    gpio_map.insert(14, Arc::new(Mutex::new(pins.gpio14.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(15, Arc::new(Mutex::new(pins.gpio15.into())));
    gpio_map.insert(16, Arc::new(Mutex::new(pins.gpio16.into())));
    gpio_map.insert(17, Arc::new(Mutex::new(pins.gpio17.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(18, Arc::new(Mutex::new(pins.gpio18.into())));
    gpio_map.insert(19, Arc::new(Mutex::new(pins.gpio19.into())));
    // gpio_map.insert(20, Arc::new(Mutex::new(pins.gpio20.into())));
    gpio_map.insert(21, Arc::new(Mutex::new(pins.gpio21.into())));
    gpio_map.insert(22, Arc::new(Mutex::new(pins.gpio22.into())));
    #[cfg(not(feature = "display"))]
    gpio_map.insert(23, Arc::new(Mutex::new(pins.gpio23.into())));
    // gpio_map.insert(27, Arc::new(Mutex::new(pins.gpio27.into())));

    #[cfg(feature = "display")]
    use esp_idf_hal::spi::SpiDriver;

    #[cfg(feature = "display")]
    let mut pcd_rst = pins.gpio2;
    #[cfg(feature = "display")]
    let mut pcd_dc = pins.gpio4;
    #[cfg(feature = "display")]
    let mut pcd_light = pins.gpio13;
    #[cfg(feature = "display")]
    let mut pcd_cs = pins.gpio15;
    #[cfg(feature = "display")]
    let mut pcd_clk = pins.gpio18;
    #[cfg(feature = "display")]
    let mut pcd_din = pins.gpio23;

    #[cfg(feature = "display")]
    let dc_pin_driver = PinDriver::input_output(pcd_dc).expect("Get dc pin driver");
    #[cfg(feature = "display")]
    let reset_pin_driver = PinDriver::input_output(pcd_rst).expect("Get reset pin driver");
    #[cfg(feature = "display")]
    let light_pin_driver = PinDriver::input_output(pcd_light).expect("Get light pin driver");
    #[cfg(feature = "display")]
    let cs_pin_driver = PinDriver::input_output(pcd_cs).expect("cs light pin driver");
    #[cfg(feature = "display")]
    let clk_pin_driver = PinDriver::input_output(pcd_clk).expect("cs light pin driver");
    #[cfg(feature = "display")]
    let din_pin_driver = PinDriver::input_output(pcd_din).expect("cs light pin driver");
    #[cfg(feature = "display")]
    let mut pcd = pcd8544::PCD8544::new(
        clk_pin_driver,
        din_pin_driver,
        dc_pin_driver,
        cs_pin_driver,
        reset_pin_driver,
        light_pin_driver,
    )
    .expect("Works");
    #[cfg(feature = "display")]
    pcd.reset().expect("reset");

    #[cfg(feature = "display")]
    sleep(Duration::from_millis(250));

    let nvs_part = match EspNvs::new(nvs.clone(), WIFI_NAMESPACE, true) {
        Ok(nvs) => {
            println!("Got namespace {:?} from default partition", WIFI_NAMESPACE);
            nvs
        }
        Err(e) => panic!("Could't get namespace {:?}", e),
    };

    #[cfg(feature = "motor")]
    let in1 = PinDriver::output(pins.gpio26).unwrap();
    #[cfg(feature = "motor")]
    let in2 = PinDriver::output(pins.gpio25).unwrap();
    #[cfg(feature = "motor")]
    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default().frequency(25.kHz().into()),
    )
    .expect("Get timer");
#[cfg(feature = "motor")]
    let driver = LedcDriver::new(peripherals.ledc.channel0, timer_driver, pins.gpio27)
        .expect("get ledc driver");
    #[cfg(feature = "motor")]
    let mut motor = l298n::Motor::new(in1, in2, driver);

    #[cfg(feature = "motor")]
    motor.forward();
    #[cfg(feature = "motor")]
    motor.set_speed(256); // Adjust duty cycle

    #[cfg(feature = "servo")]
    let servo_cfg = ServoConfig {
        // below are servo magic values that i basically guessed except for frequency and pin. /shrug
        max_angle: 180,
        min_width_us: 500,
        max_width_us: 2500,
        frequency: 50,
        timer_number: 0,
        pin: 14,
        channel: 0,
        speed_mode: 0,
    };
    #[cfg(feature = "servo")]
    let servo = Servo::init(servo_cfg).expect("Initialized Servo");

    #[cfg(feature = "servo")]
    {
        for i in 0..180 {
            servo.write_angle(i as f64).expect("wrote angle");

            FreeRtos::delay_ms(20);

            println!("Angle: {}", servo.read_angle().expect("angle read"));
        }
        servo.write_angle(0 as f64).expect("wrote angle");
    }
    const MAX_STR_LEN: usize = 100;
    const MAX_KEY_LEN: usize = 500;
    #[cfg(feature = "reset_volatile")]
    {
        nvs_part.remove(TAG_SSID_STR).unwrap_or_default();
        nvs_part.remove(TAG_KEY_STR).unwrap_or_default();
    }
    let mut buffer: [u8; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let ssid_str: &str = match nvs_part.get_str(TAG_SSID_STR, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_KEY_LEN] = [0; MAX_KEY_LEN];
    let wifi_key_str: &str = match nvs_part.get_str(TAG_KEY_STR, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let device_name: &str = match nvs_part.get_str(DEVICE_NAME_STR, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_KEY_LEN] = [0; MAX_KEY_LEN];
    let device_location: &str = match nvs_part.get_str(DEVICE_LOCATION_STR, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let mqtt_url: &str = match nvs_part.get_str(MQTT_URL_STR, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let mqtt_user: &str = match nvs_part.get_str(MQTT_USER, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };

    let mut buffer: [u8; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let mqtt_password: &str = match nvs_part.get_str(MQTT_PASSWORD, &mut buffer).unwrap() {
        Some(v) => v,
        None => "",
    };
    let mut buffer: [u8; MAX_KEY_LEN] = [0; MAX_KEY_LEN];
    let mqtt_enabled: bool = match nvs_part.get_str(MQTT_ENABLED_STR, &mut buffer).unwrap() {
        Some(v) => {
            println!("{v}");
            v == "on"
        }
        None => false,
    };
    let nvs_part_mutex = Arc::new(Mutex::new(nvs_part));
    let wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs)).expect("ESP wifi");
    let mutex_wifi = Arc::new(Mutex::new(wifi));

    let access_points = networking::scan(&mutex_wifi);

    let mut features = vec![];

    #[cfg(feature = "bare")]
    features.push(iot_common::BARE);

    #[cfg(feature = "temp_humidity")]
    features.push(iot_common::TEMPERATURE_HUMIDITY);

    #[cfg(feature = "temp_pressure")]
    features.push(iot_common::TEMPERATURE_PRESSURE);

    #[cfg(feature = "servo")]
    features.push(iot_common::SERVO);

    #[cfg(feature = "motor")]
    features.push(iot_common::MOTOR);

    #[cfg(feature = "display")]
    features.push(iot_common::DISPLAY);

    #[cfg(feature = "motion_sensor")]
    features.push(iot_common::MOTION_SENSOR);

    #[cfg(feature = "water_sensor")]
    features.push(iot_common::WATER_SENSOR);

    #[cfg(feature = "reed_switch")]
    features.push(iot_common::REED_SWITCH);

    #[cfg(feature = "air_quality")]
    features.push(iot_common::AIR_QUALITY);

    let mut mdns = esp_idf_svc::mdns::EspMdns::take().expect("Take Mdns");
    let product_string = get_features(features.clone());
    let model = device_config.model.clone();
    let model_id = device_config.id.clone();
    mdns::register_service(
        &mut mdns,
        model.clone(),
        product_string.clone(),
        model_id.clone(),
        device_name.to_owned(),
        device_location.to_owned(),
    );

    if ssid_str.len() == 0 {
        networking::start_ap(&mutex_wifi, model.clone(), model_id.clone());
        let np = nvs_part_mutex.clone();

        let httpserver = ap_routes::start_http(
            access_points.to_owned(),
            ssid.clone(),
            password.clone(),
            ssid_found.clone(),
            np,
        );

        // Handle requests until the SSID/Password were supplied via HTTP server before moving on
        println!("Waiting for SSID");
        loop {
            let found_ssid: bool;
            {
                let ssid_found_mut = ssid_found
                    .lock()
                    .unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
                found_ssid = (*ssid_found_mut).clone();
            }

            if !found_ssid {
                FreeRtos::delay_ms(1000);
            } else {
                break;
            }
        }

        drop(httpserver);
        FreeRtos::delay_ms(1000);
        let mut wifi = mutex_wifi
            .lock()
            .unwrap_or_else(|e| panic!("{}: {:?}", "Mutex worked", e));
        wifi.stop().expect("Stopped");
        while wifi.is_started().unwrap_or_default() {
            println!("not stopping");
            FreeRtos::delay_ms(150);
        }
        let _s = common::ScopeCall {
            c: || -> () {
                FreeRtos::delay_ms(2000);
                esp_idf_svc::hal::reset::restart()
            },
        };
    } else {
        let mut ssid = ssid
            .lock()
            .unwrap_or_else(|e| panic!("{}: {:?}", "Get ssid mutex lock", e));
        *ssid = ssid_str.to_string();

        let mut wifi_key = password
            .lock()
            .unwrap_or_else(|e| panic!("{}: {:?}", "Get ssid mutex lock", e));
        *wifi_key = wifi_key_str.to_string();
    }

    networking::transition_ap_to_client(&ssid, &password, &mutex_wifi);

    let data = Arc::new(Mutex::new(HashMap::new()));
    let timestamp = Arc::new(Mutex::new(0));
    let ts = timestamp.clone();
    let dc = data.clone();

    FreeRtos::delay_ms(3000);
    let user_config = Arc::new(Mutex::new(UserConfig {
        mqtt_url: mqtt_url.to_string(),
        mqtt_enabled,
        mqtt_user: mqtt_user.to_string(),
        mqtt_password: mqtt_password.to_string(),
        device_name: device_name.to_string(),
        device_location: device_location.to_string(),
        wifi_ssid: ssid_str.to_string(),
        wifi_key: wifi_key_str.to_string(),
    }));
    let config_arc = Arc::new(Mutex::new(device_config.clone()));
    let config_connected_clone = config_arc.clone();
    let np = nvs_part_mutex.clone();

    let mut _httpserver = connected_routes::start_http(
        config_connected_clone,
        user_config,
        ts,
        dc,
        access_points,
        np,
    );

    #[cfg(feature = "servo")]
    servo::create_routes(Arc::new(Mutex::new(servo)), &mut _httpserver);

    // #[cfg(feature = "motor")]
    // motor::create_routes(Arc::new(Mutex::new(motor)), &mut _httpserver);

    FreeRtos::delay_ms(1000);

    let ntp = EspSntp::new_default().unwrap();
    println!("Synchronizing with NTP Server");
    while ntp.get_sync_status() != esp_idf_svc::sntp::SyncStatus::Completed {
        sleep(Duration::from_millis(250))
    }
    println!("Time Sync Completed");

    let mut qr = Vec::with_capacity(10);
    let query_srv = mdns
        .query_srv(
            iot_common::HUB_MQTT_DNS,
            "_mqtt",
            "_tcp",
            Duration::from_secs(10),
            &mut qr,
        )
        .expect("Get results");
    println!("{query_srv}\n");
    for q in qr {
        let hostn = q.addr.first().expect("has ip");
        let port = q.port;
        println!("{hostn}:{port}\n");
    }
    let product_string = get_features(features.clone());
    let mut derived_instance_name = format!(
        "{}_{}_{}",
        model.clone(),
        product_string.clone(),
        model_id.clone()
    );

    let mut derived_instance_location = "home";

    if device_location.len() > 0 {
        derived_instance_location = device_location;
    }
    if device_name.len() > 0 {
        derived_instance_name = device_name.to_string();
    }

    use esp_idf_hal::prelude::*;

    #[cfg(feature = "temp_pressure")]
    use bmp280_driver::BMP280;

    #[cfg(feature = "temp_pressure")]
    let bmp280_config = I2cConfig::new().baudrate(400.kHz().into());

    #[cfg(feature = "temp_pressure")]
    let mut temp_pressure_sda = gpio_map[&device_config.features["temp_pressure"].pins["sda"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "temp_pressure")]
    let mut temp_pressure_scl = gpio_map[&device_config.features["temp_pressure"].pins["scl"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "temp_pressure")]
    let i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        temp_pressure_sda,
        temp_pressure_scl,
        &bmp280_config,
    )
    .expect("Established I2c");

    #[cfg(feature = "temp_pressure")]
    let mut bmp280 = BMP280::new(i2c, 0x76).expect("Failed to initialize BMP280");

    #[cfg(feature = "temp_pressure")]
    bmp280.set_control(bmp280_driver::Control {
        osrs_t: bmp280_driver::Oversampling::x1,
        osrs_p: bmp280_driver::Oversampling::x1,
        mode: bmp280_driver::PowerMode::Normal,
    });

    #[cfg(feature = "temp_humidity")]
    let mut temp_humidity_pin = gpio_map[&device_config.features["temp_humidity"].pins["sda"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "motion_sensor")]
    let mut motion_sensor_pin = gpio_map[&device_config.features["motion_sensor"].pins["sda"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "reed_switch")]
    let mut reed_switch_pin = gpio_map[&device_config.features["reed_switch"].pins["sda"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "air_quality")]
    let config = I2cConfig::new();

    #[cfg(feature = "air_quality")]
    let air_quality_scl = gpio_map[&device_config.features["air_quality"].pins["scl"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "air_quality")]
    let air_quality_sda = gpio_map[&device_config.features["air_quality"].pins["sda"]]
        .lock()
        .expect("Works");

    #[cfg(feature = "air_quality")]
    let i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c1,
        air_quality_sda,
        air_quality_scl,
        &config,
    )
    .expect("get i2c drive");

    #[cfg(feature = "air_quality")]
    let aq_config = &device_config.clone();

    #[cfg(feature = "air_quality")]
    let air_quality_device = aq_config.features["air_quality"].clone();

    #[cfg(feature = "air_quality")]
    let air_quality_address = air_quality_device
        .address
        .as_ref()
        .expect("Address configured correctly")
        .clone();

    #[cfg(feature = "air_quality")]
    let air_quality_address = u8::from_str_radix(&air_quality_address[2..], 16)
        .expect("Failed to parse the address as a hex value");

    #[cfg(feature = "air_quality")]
    let mut sgp = Sgp30::new(
        i2c,
        air_quality_address,
        esp_idf_svc::hal::delay::Delay::new(1000),
    );

    #[cfg(feature = "air_quality")]
    sgp.init().unwrap();

    let mut mqtt_client_option: Option<&mut EspMqttClient<'_>> = None;
    let mut mqtt_client: EspMqttClient<'_>;

    const MQTT_CLIENT_ID: &str = "test_publish";

    let mqtt_topic_name_base =
        format!("device/{model_id}/{model}/{derived_instance_name}/{derived_instance_location}");

    if mqtt_enabled {
        println!("connecting to mqtt");

        let conf = MqttClientConfiguration {
            client_id: Some(MQTT_CLIENT_ID),
            username: Some(mqtt_user),
            password: Some(mqtt_password),
            ..Default::default()
        };
        mqtt_client = EspMqttClient::new_cb(mqtt_url, &conf, |s| {
            println!("[Queue] Event: {}", s.payload());
        })
        .expect("Get mqtt client");
        mqtt_client_option = Some(&mut mqtt_client);
    }

    #[cfg(feature = "water_sensor")]
    let water_sensor_adc_pin_index = device_config.features["water_sensor"].pins["adc"];

    #[cfg(feature = "water_sensor")]
    let water_sensor_adc2 = AdcDriver::new(peripherals.adc2).expect("calibrated");

    #[cfg(feature = "water_sensor")]
    let water_sensor_adc1 = AdcDriver::new(peripherals.adc1).expect("calibrated");

    let mut p32 = pins.gpio32;
    let mut p33 = pins.gpio33;
    let mut iter = 0;
    //.write_string(display).expect("Failed to write string");

    #[cfg(feature = "display")]
    pcd.write_str("Habitat IOT")
        .expect("Failed to write string");

    #[cfg(feature = "motor")]
    motor.set_speed(512); // Adjust duty cycle
    loop {
        let data_loop_clone = data.clone();
        if iter > 10000 {
            iter = 0
        }
        FreeRtos::delay_ms(1000);
        data::set_timestamp(&timestamp);

        #[cfg(feature = "bare")]
        data::get_data_bare(&data_loop_clone);

        #[cfg(feature = "temp_humidity")]
        data::get_data_temp_humidity(&data_loop_clone, &mut temp_humidity_pin);

        #[cfg(feature = "temp_pressure")]
        data::get_data_temp_pressure(&data_loop_clone, &mut bmp280);

        #[cfg(feature = "motion_sensor")]
        data::get_data_motion_sensor(&data_loop_clone, &mut motion_sensor_pin);

        #[cfg(feature = "water_sensor")]
        if water_sensor_adc_pin_index == 32 {
            data::get_data_water_sensor(&data_loop_clone, &water_sensor_adc1, &mut p32);
        } else {
            data::get_data_water_sensor(&data_loop_clone, &water_sensor_adc1, &mut p33);
        };

        // #[cfg(feature = "water_sensor")]
        // data::get_data_water_sensor(&data_loop_clone, &water_sensor_adc, &mut water_sensor_adc_pin);

        #[cfg(feature = "reed_switch")]
        data::get_data_reed_switch(&data_loop_clone, &mut reed_switch_pin);

        #[cfg(feature = "air_quality")]
        data::get_data_air_quality(&mut sgp, &data_loop_clone);

        if mqtt_enabled {
            send_mqtt(
                &mut mqtt_client_option,
                data_loop_clone,
                &mqtt_topic_name_base,
            );
        }

        #[cfg(feature = "display")]
        {
            // if iter % 50 == 0 {
            pcd.clear().expect("cleared");
            // }
            let dmut = data.clone();
            let d = dmut.lock().expect("get data");
            for k in d.keys() {
                let val = d.get(k).expect("value exists");
                pcd.write_str(format!("{k}:\n {val}").as_str())
                    .expect("String written");
            }
        }
    }
}

fn send_mqtt(
    mqtt_client_option: &mut Option<&mut EspMqttClient<'_>>,
    data_loop_clone: Arc<Mutex<HashMap<String, String>>>,
    mqtt_topic_name_base: &String,
) {
    if let Some(client) = mqtt_client_option {
        let d = data_loop_clone.clone();
        let da = d
            .lock()
            .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));

        for k in da.keys() {
            let v = da.get(k).expect("value exists");
            let topic = format!("{mqtt_topic_name_base}/{k}/data");
            let res = match client.publish(topic.as_str(), QoS::AtMostOnce, false, v.as_bytes()) {
                Ok(r) => r,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };
        }
    }
}

pub fn get_features(features: Vec<&str>) -> String {
    if features.len() == 1 {
        return features.first().expect("Get first").to_string();
    }
    let mut str = String::default();
    for p in features {
        let pstr = p.to_string();
        str = format!("{str}_{pstr}").to_string();
    }
    str
}
