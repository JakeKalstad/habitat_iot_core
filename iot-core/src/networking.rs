use std::sync::{Arc, Mutex};

use embedded_svc::wifi::{AccessPointConfiguration, AccessPointInfo, ClientConfiguration, Configuration};
use esp_idf_svc::{hal::delay::FreeRtos, wifi::EspWifi};

pub fn transition_ap_to_client(
    ssid: &Mutex<String>,
    password: &Mutex<String>,
    mutex_wifi: &Arc<Mutex<EspWifi<'_>>>,
) {
    let ssid_mut = ssid.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
    let password_mut = password.lock().unwrap_or_else(|e| panic!("{}: {:?}", "locked mutex", e));
    let mut wifi = mutex_wifi.lock().unwrap_or_else(|e| panic!("{}: {:?}", "Mutex worked", e));
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid_mut.as_str().try_into().unwrap(),
        password: password_mut.as_str().try_into().unwrap(),
        ..Default::default()
    });
    log::info!("Set wifi configuration");
    wifi.set_configuration(&wifi_configuration)
        .expect("Wifi configuration set");
    
    // wifi.wait_netif_up().expect("Wait netif up");
    wifi.connect().expect("Wifi Connected");
    log::info!("Wifi connected");

    while !wifi.is_connected().unwrap() {
        FreeRtos::delay_ms(150);
    }
    log::info!("Wifi connected");
}

pub fn start_ap(mutex_wifi: &Arc<Mutex<EspWifi<'_>>>, model: String, id: String) {
    let mut wifi = mutex_wifi.lock().unwrap_or_else(|e| panic!("{}: {:?}", "Mutex worked", e));
    wifi.start().expect("started");
    while !wifi.is_started().unwrap_or_default() {
        println!("wifi not started");
        FreeRtos::delay_ms(150);
    }
    
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: format!("{}-{model}-{id}", iot_common::AP_PREFIX).as_str().try_into().unwrap(),
        password: "test".try_into().unwrap(),
        ..Default::default()
    }))
    .expect("Set Access Point");

    log::info!("Waiting for DHCP lease...");
    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        log::info!("start AP Waiting for station {:?}", config);
        FreeRtos::delay_ms(150);
    }
    log::info!("Wifi connected");
}

pub fn scan(mutex_wifi: &Arc<Mutex<EspWifi<'_>>>) -> Vec<AccessPointInfo> {
    let mut wifi = mutex_wifi.lock().unwrap_or_else(|e| panic!("{}: {:?}", "Mutex worked", e));
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ..Default::default()
    }))
    .expect("Set Client");
    wifi.start().expect("Started");
    while !wifi.is_started().unwrap_or_default() {
        println!("wifi not started");
        FreeRtos::delay_ms(150);
    }
    let access_points = wifi.scan().expect("Scanning access points");
    wifi.disconnect().expect("disconnect from scan");
    while wifi.is_connected().unwrap_or_default() {
        println!("wifi not connected");
        FreeRtos::delay_ms(150);
    }
    println!("{:?}", access_points);
    access_points
}
