use libmdns;
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    packet::v5::reason_codes::ReasonCode,
    utils::rng_generator::CountingRng,
};
use std::{
    net::ToSocketAddrs, time::Duration
};
use tokio::net::TcpStream;
use uuid;
use dotenv;

use embedded_io_adapters::tokio_1::FromTokio;

struct DeviceConfiguration {
    id: String,
    model: String,
    product: String,
    device_name: String,
    device_location: String,
}
struct Execution {
    device: Vec<DeviceConfiguration>,
}

fn get_location() -> String {
    let vs = vec![
        "Bedroom",
        "Living Room",
        "Upstairs Livingroom",
        "Bathroom",
        "Spare Room",
        "Back Patio",
    ];
    vs.choose(&mut rand::thread_rng())
        .unwrap_or(&"rand-error")
        .to_string()
}

fn get_product() -> String {
    let vs = vec![
        iot_common::BARE,
        iot_common::WATER_SENSOR,
        iot_common::AIR_QUALITY,
        iot_common::REED_SWITCH,
        iot_common::MOTION_SENSOR,
        iot_common::TEMPERATURE_HUMIDITY,
    ];
    vs.choose(&mut rand::thread_rng())
        .unwrap_or(&"rand-error")
        .to_string()
}

fn get_model() -> String {
    let vs = vec!["test-model", "test2-model", "unknown", "esp32-wroom"];
    vs.choose(&mut rand::thread_rng())
        .unwrap_or(&"rand-error")
        .to_string()
}

fn get_device_name() -> String {
    let vs = vec!["my-thing", "smarty_thing", "new_xmas_gift", "testo", "fun"];
    let first = vs
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"rand-error")
        .to_string();

    let vs = vec!["iot", "device", "sensor", "detectoid", "embed"];
    first + vs.choose(&mut rand::thread_rng()).unwrap_or(&"rand-error")
}

fn get_message(product: String) -> String {
    let val = rand::thread_rng().gen_range(35.0..=100.0);
    let val2 = rand::thread_rng().gen_range(15.0..=70.0);
    let bool_state = rand::thread_rng().gen_bool(0.50);
    if product == iot_common::TEMPERATURE_HUMIDITY {
        let json = serde_json::to_string(&iot_common::TemperatureHumidityResult{temperature: val, humidity: val2}).expect("unmarhsl");
        return json;
    }

    if product == iot_common::AIR_QUALITY {
        let json = serde_json::to_string(&iot_common::AirQualityResult{co2_ppm: val as u16, tvoc_ppb: val2 as u16}).expect("unmarhsl");
        return json;
    }
    if product == iot_common::MOTION_SENSOR {
        let json = serde_json::to_string(&iot_common::BoolResult{state: bool_state}).expect("unmarhsl");
        return json;
    }
    if product == iot_common::WATER_SENSOR {
        let json = serde_json::to_string(&iot_common::SingleResult{value: val as u16}).expect("unmarhsl");
        return json;
    }
    if product == iot_common::REED_SWITCH {
        let json = serde_json::to_string(&iot_common::BoolResult{state: bool_state}).expect("unmarhsl");
        return json;
    }
    let random_message = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(15)
    .map(char::from)
    .collect();

    let json = serde_json::to_string(&iot_common::BareResult{message: random_message}).expect("unmarhsl");
    json
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let mqtt_host = std::env::var("MOSQUITTO_HOST")
        .expect("Missing `MOSQUITTO_HOST` env variable, needed for running the server");
    let mqtt_port = std::env::var("MOSQUITTO_PORT")
        .expect("Missing `MOSQUITTO_PORT` env variable, needed for running the server");
    let mqtt_user = std::env::var("MOSQUITTO_USER")
        .unwrap_or_default();
    let mqtt_password = std::env::var("MOSQUITTO_PASSWORD")
        .unwrap_or_default();
    let device_count = std::env::var("DEVICE_COUNT")
        .unwrap_or_default();
    let device_count:i32 = device_count.parse().unwrap_or_default();
    let config = if device_count > 1 { 
        let mut devices = vec![];
        for _n in 1..device_count {
            devices.push(DeviceConfiguration {
                id: uuid::Uuid::new_v4().to_string(),
                device_location: get_location(),
                device_name: get_device_name(),
                model: get_model(),
                product: get_product(),
            });
        }
        Execution { device: devices }
    } else {
        let uuid = uuid::Uuid::new_v4().to_string();
        Execution {
            device: vec![DeviceConfiguration {
                id: uuid,
                device_location: get_location(),
                device_name: "Dummy test".to_string(),
                model: "test-model".to_string(),
                product: iot_common::BARE.to_string(),
            }],
        }
    };

    let mqtt_port: u16 = mqtt_port.parse().expect("get mqtt port");
    for device in config.device {
            let responder = libmdns::Responder::new().unwrap();
            let n_host = mqtt_host.clone();
            let n_user = mqtt_user.clone();
            let n_pass = mqtt_password.clone();
            
        tokio::task::spawn(async move {
            // let responder = libmdns::Responder::new().unwrap();
            let id = device.id;
            let model = device.model;
            let product = device.product;
            let device_name = device.device_name;
            let device_location = device.device_location;

            let service_address = (n_host, mqtt_port);
            
            let addr = service_address
                .to_socket_addrs()
                .expect("get address")
                .next()
                .ok_or("Unable to resolve domain")
                .expect("retrieve socket address for mqtt");

            let connection = TcpStream::connect(addr)
                .await
                .map_err(|_| ReasonCode::NetworkError)
                .unwrap();
            let connection = FromTokio::<TcpStream>::new(connection);
            let mut config = ClientConfig::new(
                rust_mqtt::client::client_config::MqttVersion::MQTTv5,
                CountingRng(20000),
            );
            if n_user.len() > 0 {
                config.add_username(&n_user);
                config.add_password(&n_pass);
            }
            config.add_max_subscribe_qos(
                rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1,
            );

            config.add_client_id(&id);
            config.max_packet_size = 1000;
            let mut recv_buffer = [0; 800];
            let mut write_buffer = [0; 800];

            let mut client = MqttClient::<_, 5, _>::new(
                connection,
                &mut write_buffer,
                800,
                &mut recv_buffer,
                800,
                config,
            );

            client.connect_to_broker().await.unwrap();
            let _mqtt_svc = responder.register(
                "_habitat_iot._tcp".to_owned(),
                "habitat_iot-".to_owned() + &id,
                80,
                &[
                    &format!("model={model}"),
                    &format!("product={product}"),
                    &format!("device_name={device_name}"),
                    &format!("device_location={device_location}"),
                ],
            );
            
            loop {
                let msg = &get_message(product.clone());
                client
                    .send_message(
                        &format!(
                        "device/{id}/{model}/{device_name}/{device_location}/{product}/data"
                    ),
                        msg.as_bytes(),
                        rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS0,
                        true,
                    )
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(2500)).await;
            }
        });
    }

    loop {
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
}
