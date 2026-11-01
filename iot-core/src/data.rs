use esp_idf_hal::gpio::AnyIOPin;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "temp_humidity", feature = "water_sensor"))]
use esp_idf_svc::hal::gpio::PinDriver;

#[cfg(feature = "bare")]
pub fn get_data_bare(data: &Arc<Mutex<HashMap<String, String>>>) {
    let data = data.clone();
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::BARE.to_string(),
        serde_json::to_string(&iot_common::BareResult {
            message: "Here and happy".to_string(),
        })
        .expect("Marshal to string"),
    );
}

#[cfg(feature = "temp_pressure")]
use bmp280_driver::BMP280;

#[cfg(feature = "temp_pressure")]
use esp_idf_hal::delay;

#[cfg(feature = "temp_pressure")]
use esp_idf_hal::i2c::I2cDriver;

#[cfg(feature = "temp_pressure")]
pub fn get_data_temp_pressure(
    data: &Arc<Mutex<HashMap<String, String>>>,
    bmp280: &mut BMP280<I2cDriver<'_>>,
) {
    let data = data.clone();
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::TEMPERATURE_PRESSURE.to_string(),
        serde_json::to_string(&iot_common::TemperaturePressureResult {
            temperature: bmp280.temp(),
            pressure: bmp280.pressure() / 100_f64,
        })
        .expect("Marshal to string"),
    );

    bmp280.set_control(bmp280_driver::Control {
        osrs_t: bmp280_driver::Oversampling::x1,
        osrs_p: bmp280_driver::Oversampling::x1,
        mode: bmp280_driver::PowerMode::Normal,
    });
}
#[cfg(feature = "temp_humidity")]
pub fn get_data_temp_humidity(
    data: &Arc<Mutex<HashMap<String, String>>>,
    pin: impl esp_idf_hal::peripheral::Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin>,
) {
    use dht_sensor::{dht11, DhtReading};
    use esp_idf_hal::delay::{self, FreeRtos};

    FreeRtos::delay_ms(1500);

    let mut dht11_pin = PinDriver::input_output(pin).unwrap();
    dht11_pin.set_high().unwrap();
    println!("get measurement");

    match dht11::Reading::read(&mut delay::Ets, &mut dht11_pin) {
        Ok(r) => {
            println!("measure: {r:?}");
            let data = data.clone();
            let mut da = data
                .lock()
                .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
            da.insert(
                iot_common::TEMPERATURE_HUMIDITY.to_string(),
                serde_json::to_string(&iot_common::TemperatureHumidityResult {
                    humidity: r.relative_humidity as f32 / 10.0,
                    temperature: r.temperature as f32 / 10.0,
                })
                .expect("Marshal to string"),
            );
        }
        Err(e) => println!("Failed to retrieve information: {:?}", e),
    }
}

#[cfg(feature = "motion_sensor")]
pub fn get_data_motion_sensor(data: &Arc<Mutex<HashMap<String, String>>>,
    pin: impl esp_idf_hal::peripheral::Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin>) {
    let motion_pin = PinDriver::input(pin).unwrap();
    let data = data.clone();
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::MOTION_SENSOR.to_string(),
        serde_json::to_string(&iot_common::BoolResult {
            state: motion_pin.is_high(),
        })
        .expect("marshal"),
    );
}

#[cfg(feature = "reed_switch")]
pub fn get_data_reed_switch(data: &Arc<Mutex<HashMap<String, String>>>, 
    pin: impl esp_idf_hal::peripheral::Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin>) {
    let reed_switch_pin = PinDriver::input(pin).unwrap();
    let data = data.clone();
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::REED_SWITCH.to_string(),
        serde_json::to_string(&iot_common::BoolResult {
            state: reed_switch_pin.is_high(),
        })
        .expect("marshal"),
    );
}

#[cfg(feature = "air_quality")]
pub fn get_data_air_quality(
    sgp: &mut sgp30::Sgp30<esp_idf_svc::hal::i2c::I2cDriver<'_>, esp_idf_svc::hal::delay::Delay>,
    data: &Arc<Mutex<HashMap<String, String>>>,
) {
    let measurement: sgp30::Measurement = sgp.measure().unwrap();
    let data = data.clone();
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::AIR_QUALITY.to_string(),
        serde_json::to_string(&iot_common::AirQualityResult {
            co2_ppm: measurement.co2eq_ppm,
            tvoc_ppb: measurement.tvoc_ppb,
        })
        .expect("Marshal to string"),
    );
}

pub fn set_timestamp(timestamp: &Arc<Mutex<u128>>) {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_millis();
    let mut ts = timestamp
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    *ts = in_ms;
}

#[cfg(feature = "water_sensor")]
pub fn get_data_water_sensor<
    AdcType: esp_idf_hal::adc::Adc, // ADC1 or ADC2 type
    P: esp_idf_hal::gpio::ADCPin<Adc = AdcType>, // Pin must implement ADCPin for the chosen ADC
>(
    data: &Arc<Mutex<HashMap<String, String>>>,
    adc: &esp_idf_hal::adc::oneshot::AdcDriver<'_, AdcType>, // Use the generic AdcType here
    adc_pin: &mut P, // Pin is of type P, which is specific to the AdcType
) {
    use esp_idf_hal::adc::oneshot::AdcChannelDriver;

    let config = esp_idf_hal::adc::oneshot::config::AdcChannelConfig {
        attenuation: esp_idf_hal::adc::attenuation::DB_11,
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(adc, adc_pin, &config).expect("find drive");
    let water_sensor_value = adc.read(&mut adc_pin).expect("woop");
    println!("sensor: {water_sensor_value}");
    let mut da = data
        .lock()
        .unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
    da.insert(
        iot_common::WATER_SENSOR.to_string(),
        serde_json::to_string(&iot_common::SingleResult {
            value: water_sensor_value,
        })
        .expect("marshal"),
    );
}
