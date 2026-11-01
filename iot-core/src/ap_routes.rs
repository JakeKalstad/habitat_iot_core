use embedded_svc::{http::Headers, wifi::AccessPointInfo};
use esp_idf_svc::{hal::{delay::FreeRtos, reset::restart}, http::{server::EspHttpServer, Method}, io::{Read, Write}, nvs::{EspNvs, NvsDefault}};
use std::sync::{Arc, Mutex};
use crate::common;

use crate::{TAG_KEY_STR, TAG_SSID_STR};
static INDEX_HTML: &str = include_str!("html/index.html");
static SUCCESS_HTML: &str = include_str!("html/success.html");

pub fn start_http(
    access_points: Vec<AccessPointInfo>,
    ssid: Arc<Mutex<String>>,
    password: Arc<Mutex<String>>,
    ssid_found: Arc<Mutex<bool>>,
    nvs_part: Arc<Mutex<EspNvs<NvsDefault>>>,
) -> EspHttpServer<'static> {
    let http_config = &esp_idf_svc::http::server::Configuration::default();
    let mut httpserver = EspHttpServer::new(http_config).expect("Create http serve");

    // Use `Arc` clones to move into the closure
    let access_points_clone = access_points.clone();
    httpserver
        .fn_handler::<anyhow::Error, _>("/", Method::Get, move |request| {
            home(request, &access_points_clone)
        })
        .expect("serve");

    httpserver
        .fn_handler::<anyhow::Error, _>("/upload", Method::Post, {
            let ssid = ssid.clone();
            let password = password.clone();
            let ssid_found = ssid_found.clone();
            let nvs_part = nvs_part.clone();
            move |request| post_network_information(request, &ssid, &password, &ssid_found, &nvs_part)
        })
        .expect("serve");

    httpserver
}

fn home(
    request: embedded_svc::http::server::Request<
        &mut esp_idf_svc::http::server::EspHttpConnection<'_>,
    >,
    access_points: &Vec<AccessPointInfo>,
) -> Result<(), anyhow::Error> {
    let mut response = request.into_ok_response()?;
    let mut list = "<select name=\"ssid\">".to_string();
    for ap in access_points {
        list += &format!(
            "<option value=\"{}\">{} :: 🛜 {}</option>",
            ap.ssid, ap.ssid, ap.signal_strength
        );
    }
    list += "</select>";
    let mut html = INDEX_HTML.to_string();
    html = html.replace("{OPTIONS}", &list);
    response.write(html.as_bytes())?;
    Ok(())
}

fn post_network_information(
    mut request: embedded_svc::http::server::Request<
        &mut esp_idf_svc::http::server::EspHttpConnection<'_>,
    >,
    ssid: &Mutex<String>,
    password: &Mutex<String>,
    ssid_found: &Mutex<bool>,
    np: &Arc<Mutex<EspNvs<NvsDefault>>>,
) -> Result<(), anyhow::Error> {
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
    let np_clone = np.clone();
    let mut nvs_part = np_clone.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked NVS mutex", e));
    for k in form_data {
        match k.0.as_str() {
            "ssid" => {
                let mut ssid_mut = ssid.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
                *ssid_mut = k.1.clone();
                match nvs_part.set_str(TAG_SSID_STR, k.1.clone().trim_matches(char::from(0))) {
                    Ok(_v) => println!("{:?} updated ssid", k.1.clone()),
                    Err(e) => println!("{:?} not updated ssid {:?}", k.1.clone(), e),
                };
            }
            "password" => {
                let mut password_mut = password.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
                *password_mut = k.1.clone();
                match nvs_part.set_str(TAG_KEY_STR, k.1.clone().trim_matches(char::from(0))) {
                    Ok(_v) => println!("{:?} updated key", k.1.clone()),
                    Err(e) => println!("{:?} not updated key {:?}", k.1.clone(), e),
                };
            }
            _ => {}
        }
    }
    let mut ssid_found_mut = ssid_found.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
    *ssid_found_mut = true;
    let mut response = request.into_ok_response()?;
    response.write(SUCCESS_HTML.as_bytes())?;


    let _s = common::ScopeCall {
        c: || -> () {
            FreeRtos::delay_ms(2000);
            restart()
        },
    };
    Ok(())
}