use build_time::build_time_local;

pub fn register_service(
    mdns: &mut esp_idf_svc::mdns::EspMdns,
    model: String,
    product: String,
    id: String,
    device_name: String,
    device_location: String,
) {
    let name = format!("{}-{id}", iot_common::HUB_HOST);
    let homename = format!(
        "{}-{}",
        iot_common::HUB_HOST,
        id
    );
    mdns.set_hostname(homename).expect("Setting host name");
    mdns.add_service(
        Some(name.clone().as_str()),
        iot_common::SERVICE_TYPE,
        "_tcp",
        80,
        &[
            ("build-time", build_time_local!()),
            ("model", model.as_str()),
            ("product", product.as_str()),
            ("device_name", &device_name),
            ("device_location", &device_location),
        ],
    )
    .expect("MDNS ADDED SERVICE");
}
