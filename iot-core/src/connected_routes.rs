use embedded_svc::http::Headers;
use embedded_svc::wifi::AccessPointInfo;
use esp_idf_svc::{
    hal::{delay::FreeRtos, reset::restart},
    http::{server::EspHttpServer, Method},
    io::{Read, Write},
    nvs::{EspNvs, NvsDefault},
};
use urlencoding::decode;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    common, configuration::UserConfig, DEVICE_LOCATION_STR, DEVICE_NAME_STR, MQTT_ENABLED_STR, MQTT_PASSWORD, MQTT_URL_STR, MQTT_USER, TAG_KEY_STR, TAG_SSID_STR
};
static ADMIN_HTML: &str = include_str!("html/admin.html");
static UPDATE_SUCCESS_HTML: &str = include_str!("html/network_update_success.html");
pub fn start_http(
    cf: Arc<Mutex<iot_common::Device>>,
    uc: Arc<Mutex<UserConfig>>,
    ts: Arc<Mutex<u128>>,
    dc: Arc<Mutex<HashMap<String, String>>>,
    access_points: Vec<AccessPointInfo>,
    np: Arc<Mutex<EspNvs<NvsDefault>>>,
) -> EspHttpServer<'static> {
    let http_config = &esp_idf_svc::http::server::Configuration::default();
    let mut httpserver = EspHttpServer::new(http_config).expect("Create http serve");

    let user_config_clone = uc.clone();
    let user_config_heartbeat_clone = uc.clone();

    let config_clone = cf.clone();
    let config_clone_home = cf.clone();

    let nvs_part_clone = np.clone();
    let nvs_part_set_mqtt_clone = np.clone();
    let nvs_part_clear_network_clone = np.clone();
    let nvs_part_set_network = np.clone();

    httpserver
        .fn_handler::<anyhow::Error, _>("/", Method::Get, move |request| {
            let mut response = request.into_ok_response()?;
            let mut list = "<select name=\"ssid\">".to_string();
            let user_config = &user_config_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "get user_config", e));
            for ap in &access_points {
                let mut selected = "";
                if ap.ssid.to_string() == user_config.wifi_ssid {
                    selected = "selected"
                }
                list += &format!(
                    "<option {} value=\"{}\">{} :: 🛜 {}</option>",
                    selected, ap.ssid, ap.ssid, ap.signal_strength
                );
            }
            list += "</select>";
            let mut html = ADMIN_HTML.to_string();
            html = html.replace("{OPTIONS}", &list);

            let config = &config_clone_home.lock().unwrap_or_else(|e| panic!("{}: {:?}", "get lock", e));
            let model = &config.model;
            let version = &config.version;
            let id = &config.id;

            let table = format!(
                "<table>
                <tr> 
                    <td><b>Model</b></td>
                    <td>{}</td>
                </tr>
                <tr> 
                    <td><b>ID</b></td>
                    <td>{}</td>
                </tr>
                <tr> 
                    <td><b>Version</b></td>
                    <td>{}</td>
                </tr>
                </table>",
                model, id, version
            );
            html = html.replace("{SPECS}", &table.clone());
            let mut enabled = "";
            if user_config.mqtt_enabled {
                enabled = "checked"
            }
            html = html.replace("{DEVICE_NAME}", &user_config.device_name.clone());
            html = html.replace("{DEVICE_LOCATION}", &user_config.device_location.clone());
            html = html.replace("{MQTT_URL}", &user_config.mqtt_url.clone());
            html = html.replace("{MQTT_USER}", &user_config.mqtt_user.clone());
            html = html.replace("{MQTT_PASSWORD}", &&user_config.mqtt_password.clone());
            html = html.replace("{MQTT_ENABLED}", enabled);

            html = html.replace("{SSID_KEY}", &user_config.wifi_key.clone());
            response.write(html.as_bytes())?;
            Ok(())
        })
        .expect("serve");
        
    httpserver
        .fn_handler::<anyhow::Error, _>("/heartbeat", Method::Get, move |request| {
            let user_config = &user_config_heartbeat_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "get user_config", e));
            let mqtt_enabled = user_config.mqtt_enabled.clone();
            let mut response = request.into_ok_response()?;
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let in_ms = since_the_epoch.as_millis();
            let config = &config_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "get lock", e));
            let model = config.model.clone();
            let version = config.version.clone();
            let heartbeat = iot_common::Heartbeat {
                timestamp: in_ms,
                model: model,
                version: version,
                mqtt_enabled: mqtt_enabled,
            };
            let output_json = serde_json::to_string(&heartbeat).unwrap();
            response.write(output_json.as_bytes())?;
            Ok(())
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/data", Method::Get, move |request| {
            let mut response = request.into_ok_response()?;

            let ts = ts.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked ts", e));
            let da = dc.lock().unwrap_or_else(|e| panic!("{}: {:?}", "unlocked mutex", e));
            let data = iot_common::DeviceData {
                data: da.clone(),
                timestamp: ts.clone().try_into().expect("Cast to i64"),
            };
            let output_json = serde_json::to_string(&data.clone()).unwrap();
            response.write(output_json.as_bytes())?;
            Ok(())
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/set_network", Method::Post, move |mut request| {
            let len = request.content_len().unwrap_or(0) as usize;

            if len > common::MAX_POST_LEN {
                request
                    .into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buf = vec![0; len];
            request.read_exact(&mut buf)?;
            let form_body = match String::from_utf8(buf) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let form_data = common::parse_form_data(&form_body);

            // Lock network NVS separately
            let mut nvs_part_network = nvs_part_set_network.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked network NVS", e));
            for k in form_data {
                match k.0.as_str() {
                    "ssid" => {
                        nvs_part_network
                            .set_str(TAG_SSID_STR, k.1.clone().trim_matches(char::from(0)))
                            .expect("failed to update SSID");
                    }
                    "password" => {
                        nvs_part_network
                            .set_str(TAG_KEY_STR, k.1.clone().trim_matches(char::from(0)))
                            .expect("failed to update password");
                    }
                    _ => {}
                }
            }
            let mut response = request.into_ok_response()?;
            response.write(UPDATE_SUCCESS_HTML.as_bytes())?;

            let _s = common::ScopeCall {
                c: || -> () {
                    FreeRtos::delay_ms(2000);
                    restart()
                },
            };
            Ok(())
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/set_device_data", Method::Post, move |mut request| {
            let len = request.content_len().unwrap_or(0) as usize;

            if len > common::MAX_POST_LEN {
                request
                    .into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buf = vec![0; len];
            request.read_exact(&mut buf)?;
            let form_body = match String::from_utf8(buf) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let form_data = common::parse_form_data(&form_body);

            let mut nvs_part = nvs_part_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked NVS mutex", e));
            for k in form_data {
                match k.0.as_str() {
                    "device_name" => {
                        match nvs_part
                            .set_str(DEVICE_NAME_STR, k.1.clone().trim_matches(char::from(0)))
                        {
                            Ok(_v) => println!("{:?} updated device name", k.1.clone()),
                            Err(e) => println!("{:?} not updated device name {:?}", k.1.clone(), e),
                        };
                    }
                    "device_location" => {
                        match nvs_part
                            .set_str(DEVICE_LOCATION_STR, k.1.clone().trim_matches(char::from(0)))
                        {
                            Ok(_v) => println!("{:?} updated device location", k.1.clone()),
                            Err(e) => {
                                println!("{:?} not updated device location {:?}", k.1.clone(), e)
                            }
                        };
                    }
                    _ => {}
                }
            }
            let mut response = request.into_ok_response()?;
            response.write(UPDATE_SUCCESS_HTML.as_bytes())?;

            let _s = common::ScopeCall {
                c: || -> () {
                    FreeRtos::delay_ms(2000);
                    restart()
                },
            };
            Ok(())
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/set_mqtt", Method::Post, move |mut request| {
            let len = request.content_len().unwrap_or(0) as usize;

            if len > common::MAX_POST_LEN {
                request
                    .into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buf = vec![0; len];
            request.read_exact(&mut buf)?;
            let form_body = match String::from_utf8(buf) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let form_data = common::parse_form_data(&form_body);

            let mut nvs_part = nvs_part_set_mqtt_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked NVS mutex", e));
            let mut reset_mqtt = true;
            for k in form_data {
                match k.0.as_str() {
                    "mqtt_url" => {
                        match nvs_part.set_str(
                            MQTT_URL_STR,
                            decode(k.1.clone().trim_matches(char::from(0)))
                                .expect("get str")
                                .to_string()
                                .as_str(),
                        ) {
                            Ok(_v) => println!("{:?} updated mqtt url", k.1.clone().as_str()),
                            Err(e) => println!("{:?} not updated mqtt url {:?}", k.1.clone(), e),
                        };
                    }
                    "mqtt_user" => {
                        match nvs_part.set_str(
                            MQTT_USER,
                            decode(k.1.clone().trim_matches(char::from(0)))
                                .expect("get str")
                                .to_string()
                                .as_str(),
                        ) {
                            Ok(_v) => println!("{:?} updated mqtt user", k.1.clone().as_str()),
                            Err(e) => println!("{:?} not updated mqtt user {:?}", k.1.clone(), e),
                        };
                    }
                    "mqtt_password" => {
                        match nvs_part.set_str(
                            MQTT_PASSWORD,
                            decode(k.1.clone().trim_matches(char::from(0)))
                                .expect("get str")
                                .to_string()
                                .as_str(),
                        ) {
                            Ok(_v) => println!("{:?} updated mqtt pass", k.1.clone().as_str()),
                            Err(e) => println!("{:?} not updated mqtt pass {:?}", k.1.clone(), e),
                        };
                    }
                    "mqtt_enable" => {
                        reset_mqtt = false;
                        match nvs_part
                            .set_str(MQTT_ENABLED_STR, k.1.clone().trim_matches(char::from(0)))
                        {
                            Ok(_v) => println!("{:?} updated mqtt enabled", k.1.clone()),
                            Err(e) => {
                                println!("{:?} not updated mqtt enabled {:?}", k.1.clone(), e)
                            }
                        };
                    }
                    _ => {}
                }
            }
            if reset_mqtt {
                nvs_part
                    .remove(MQTT_ENABLED_STR)
                    .expect("Remove mqtt enabled");
            }
            let mut response = request.into_ok_response()?;
            response.write(UPDATE_SUCCESS_HTML.as_bytes())?;

            let _s = common::ScopeCall {
                c: || -> () {
                    FreeRtos::delay_ms(2000);
                    restart()
                },
            };
            Ok(())
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/clear_network", Method::Post, move |request| {
            let mut nvs_part = nvs_part_clear_network_clone
                .lock()
                .expect("locked NVS mutex");

            match nvs_part.set_str(TAG_SSID_STR, "") {
                Ok(_v) => println!("cleared ssid"),
                Err(e) => println!("ssid not cleared: {:?}", e),
            };
            match nvs_part.set_str(TAG_KEY_STR, "") {
                Ok(_v) => println!("cleared key"),
                Err(e) => println!("key not cleared: {:?}", e),
            };
            let mut response = request.into_ok_response()?;
            response.write(UPDATE_SUCCESS_HTML.as_bytes())?;

            let _s = common::ScopeCall {
                c: || -> () {
                    FreeRtos::delay_ms(2000);
                    restart()
                },
            };
            Ok(())
        })
        .expect("serve");

    httpserver
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub(crate) data: String,
    pub(crate) timestamp: u128,
}
