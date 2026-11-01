# Habitat IoT

**Rust firmware and hardware designs for connected sensors and actuators, built around the ESP32.**

Habitat began in 2023 as the device side of a larger IoT platform. It brings together embedded firmware, browser-based Wi-Fi setup, local device management, sensor telemetry, and hardware designs in one project.

The original implementation predates our use of LLM-assisted development. We are releasing it so others can study the design, reuse the device interfaces, and build on the work.

**This release covers the device side. The platform it originally integrated with remains proprietary and is not included.** You can build an independent integration using the HTTP endpoints, MQTT messages, and shared Rust types described below. Access to the original platform is not supplied with this repository.

## What is here

- ESP32 firmware written in Rust, using ESP-IDF through `esp-idf-svc` and `esp-idf-hal`.
- Browser-based Wi-Fi provisioning and a local administration page.
- Persistent Wi-Fi, device, and MQTT settings in ESP32 non-volatile storage (NVS).
- Local HTTP endpoints for health information, sensor readings, and configuration.
- Optional MQTT publishing and mDNS service discovery.
- Compile-time modules for environmental sensors, digital inputs, a display, and actuators.
- Shared message types, a simulated device publisher, KiCad designs, and enclosure files.

The firmware uses ESP-IDF and its native components; this is a Rust application on ESP-IDF rather than a `no_std` implementation of the entire device stack.

## Scope and status

This is a historical engineering project being prepared for public use. The supplied snapshot includes later dependency updates and development tooling; it is not a frozen copy of the 2023 toolchain.

The code should be treated as a development starting point. A clean build and physical-device validation are still required for each board and feature combination. Some optional paths are incomplete, hardware configuration needs review, and infrastructure files retain assumptions from the original private environment.

The proprietary hub/backend, platform application, and production services are outside this release. References to them in configuration and deployment files describe the original environment, not services that this repository provides.

## Repository layout

| Path | Purpose |
| --- | --- |
| `iot-core/` | ESP32 firmware, embedded web pages, sensor acquisition, networking, and actuator code. |
| `iot-common/` | Shared serializable types, feature names, and discovery constants. |
| `fake_device/` | Host-side MQTT publisher producing simulated device readings. |
| `schematics/` | KiCad circuit and PCB designs, including environmental sensing boards. |
| `enclosure/` | Enclosure and 3D-print assets, plus reference board files. |
| `pcd8544/` | Local PCD8544 display driver dependency. |
| `kicad-rs/` | KiCad integration library and examples. |
| `devops/` | Historical build, verification, and deployment tooling. Not a standalone platform distribution. |

These are separate Cargo projects, not a single root Cargo workspace. Run commands from the relevant crate directory.

## How a device operates

1. Load the device definition embedded from `iot-core/src/config.json` and any saved user settings from NVS.
2. Scan for Wi-Fi networks. If no SSID is stored, start a setup access point and serve the provisioning page.
3. Save the submitted Wi-Fi settings and restart into client mode.
4. Join the configured network and expose the local administration and data endpoints.
5. Synchronize time with SNTP, perform the hub MQTT discovery query, and enter the acquisition loop.
6. Collect readings from the compiled-in modules and, when enabled, publish them to the configured MQTT broker.

The loop includes a one-second delay, but sensor operations add latency. This is not a guaranteed one-second sampling interval. The current startup path also waits for time synchronization before acquisition begins.

## Build and flash

### Requirements

- An ESP32 board compatible with the configured `xtensa-esp32-espidf` target, a USB data connection, and serial-port access.
- Rust with the Espressif `esp` toolchain, normally installed through `espup`, including `rust-src`.
- `ldproxy` and `espflash` available on your path.
- The native build prerequisites required by ESP-IDF and `embuild` on your host.
- A Wi-Fi network and, if using MQTT, a reachable broker.

The checked-in build configuration selects:

| Setting | Value |
| --- | --- |
| Rust toolchain | `esp` |
| Target | `xtensa-esp32-espidf` |
| MCU | `esp32` |
| ESP-IDF | `v5.2` |
| Runner | `espflash flash --monitor` |

Load the environment exported by your Espressif toolchain installation before building. Preserve the sibling `iot-common/` and `pcd8544/` directories: the firmware manifest references both through local paths.

### Start with a sensorless device

Review `iot-core/src/config.json`, assign a unique device `id`, and check the build notes below. Then, from the repository root:

```sh
cd iot-core
cargo build --release --features bare
cargo run --release --features bare
```

`cargo run` invokes the configured flash-and-monitor runner. These are the intended commands for this configuration, not a claim that this snapshot has been verified against a current toolchain.

Once the bare device works, enable the module you have wired, for example:

```sh
cargo run --release --features bare,temp_humidity
```

Features select firmware code at compile time. Adding an entry to `config.json` does not enable its driver.

## Device configuration

`iot-core/src/config.json` is embedded at build time. It describes device identity, model/version metadata, and feature wiring. Changes require a rebuild and reflash.

A minimal sensorless definition is:

```json
{
  "id": "bench-01",
  "version": "alpha",
  "model": "esp32-wroom",
  "features": {
    "bare": {
      "name": "bare",
      "description": "Sensorless development device",
      "connection_type": "none",
      "hardware_compatible": [],
      "pins": {}
    }
  }
}
```

Use a different ID for each physical device. The supplied `proto-1` value is a development identifier, not an automatically generated serial number.

Wi-Fi credentials, device name/location, and MQTT settings are configured through the device's web interface and saved separately in NVS. They are not part of this build-time JSON.

### Firmware features

| Cargo feature | Implementation / intended role |
| --- | --- |
| `bare` | Sensorless status payload: `{"message":"Here and happy"}`. |
| `temp_humidity` | DHT11 acquisition path; temperature and humidity fields. |
| `temp_pressure` | BMP280 acquisition path; temperature and pressure fields. |
| `motion_sensor` | Digital input reported as a boolean state. |
| `water_sensor` | ADC reading reported as a numeric value. |
| `reed_switch` | Digital input reported as a boolean state. |
| `air_quality` | SGP30 CO₂-equivalent and TVOC readings. |
| `display` | PCD8544 display output using the local driver. |
| `servo` | PWM servo implementation with local HTTP control. |
| `motor` | Experimental L298N motor path; HTTP route registration is commented out. |
| `reset_volatile` | Removes saved Wi-Fi SSID/key at startup; does not clear all device settings. |

Do not enable every feature together by default. Pins, buses, peripheral ownership, and driver compatibility must be checked for the selected combination. The `host` feature is declared but does not establish a host build of the ESP32 firmware.

### Wiring limitations in this snapshot

Treat the configuration as a starting point, not a verified wiring guide:

- The motion configuration names GPIO20, which is absent from the firmware's GPIO map. Its HC-SR04 label also does not match the implemented single-pin boolean acquisition path.
- Several sensor pins in the supplied JSON are absent from the current dynamic GPIO map. Reconcile the map and configuration before enabling those modules.
- The temperature/humidity path calls the DHT11 driver even though the configuration also lists DHT12. Validate the conversion/scaling against actual hardware.
- The pressure path uses a fixed `0x76` address; changing its JSON address alone does not change the driver initialization.
- Display wiring and the servo's GPIO14 configuration are set directly in `main.rs`; not every peripheral is driven by JSON pin assignments.
- The servo performs a startup sweep. Keep attached mechanisms clear during development.

## Provision and configure

1. Flash and power the device.
2. If no Wi-Fi SSID is saved, look for its `HABITAT-` prefixed setup network. The current formatter adds another hyphen after that prefix.
3. Connect and open the device's access-point gateway address in a browser. The code does not provide a captive-portal DNS redirect.
4. Choose the target network, submit its credentials, and allow the device to restart.
5. Find the device's LAN address through your router or mDNS and open `http://DEVICE_IP/`.
6. Set its name, location, MQTT broker URL, optional broker credentials, and MQTT enable checkbox as required.

The admin forms persist changes and restart the device. For a typical local broker, the URL takes the form `mqtt://BROKER_IP:1883`.

To provision again, use the admin page's network-clear action. If the device is unreachable, the `reset_volatile` build feature removes the stored SSID/key on boot. Remove that feature from subsequent builds so credentials persist normally.

## Integrate with your own backend

Your service can poll HTTP or subscribe to MQTT without implementing the proprietary platform. `iot-common/src/lib.rs` defines the shared payload structures.

### Local HTTP interface

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Provisioning page in setup mode; administration page when connected. |
| `POST` | `/upload` | Setup-mode Wi-Fi submission. |
| `GET` | `/heartbeat` | Connected-mode version, model, timestamp, and MQTT-enabled state. |
| `GET` | `/data` | Latest readings and their acquisition timestamp. |
| `POST` | `/set_network` | Save Wi-Fi settings and restart. |
| `POST` | `/set_device_data` | Save device name/location and restart. |
| `POST` | `/set_mqtt` | Save MQTT configuration and restart. |
| `POST` | `/clear_network` | Clear Wi-Fi SSID/key and restart. |
| `GET` | `/servo/coordinate` | Read the angle calculated from PWM duty, when `servo` is compiled in. |
| `POST` | `/servo/coordinate` | Set a servo destination using JSON, e.g. `{"destination":90.0}`. |

Configuration routes accept HTML form submissions. The servo readback is a calculated commanded angle, not physical position feedback.

```sh
curl http://DEVICE_IP/heartbeat
curl http://DEVICE_IP/data
```

Illustrative `/data` response:

```json
{
  "data": {
    "bare": "{\"message\":\"Here and happy\"}"
  },
  "timestamp": 1700000000000
}
```

The outer response is JSON, but each value inside `data` is itself a JSON-encoded **string**. Parse the response, then parse each feature value. Timestamps in `/data` and `/heartbeat` are Unix epoch milliseconds; MQTT feature messages do not include that wrapper timestamp.

### MQTT telemetry

The firmware publishes one message per available feature using this topic structure:

```text
device/{id}/{model}/{device_name}/{device_location}/{feature}/data
```

For example:

```text
device/bench-01/esp32-wroom/bench/lab/bare/data
```

Payload:

```json
{"message":"Here and happy"}
```

Firmware messages use QoS 0 and are not retained. If no user name is configured, the firmware derives one from model, features, and ID; the default location is `home`. Keep topic components free of `/`, `+`, and `#` when assigning names and locations.

The broker URL is taken from saved settings. Although the firmware queries the original hub's MQTT mDNS service, it does not use that query result to populate the broker configuration.

**Multi-device caveat:** the firmware currently uses the fixed MQTT client ID `test_publish`. Assign a unique client ID per device before connecting multiple boards to the same broker.

### Discovery

Devices advertise `_habitat_iot._tcp` on port 80. TXT metadata includes build time, model, enabled products/features, device name, and location. The hostname is derived as `habitat_iot-{id}`.

The firmware also queries the service instance `habitat_iot_hub_mqtt` under `_mqtt._tcp`. These names are historical integration conventions, not an included backend implementation. Use the device's IP if your network or resolver does not handle the advertised hostname.

### Payload fields

| Feature | JSON fields |
| --- | --- |
| `bare` | `message` |
| `temp_humidity` | `temperature`, `humidity` |
| `temp_pressure` | `temperature`, `pressure` |
| `motion_sensor`, `reed_switch` | `state` |
| `water_sensor` | `value` |
| `air_quality` | `co2_ppm`, `tvoc_ppb` |

The SGP30 `co2_ppm` field contains the driver's CO₂-equivalent estimate. The water sensor value is an ADC reading, not a calibrated water-level measurement. Confirm units and calibration before using readings for automation.

## Simulated devices

`fake_device/` publishes synthetic readings to an MQTT broker and advertises mDNS metadata, allowing backend integration work without a physical board. It does not implement the firmware's HTTP server.

Its manifest still references the original private Git source for `iot-common`. For an independent checkout, replace that dependency with:

```toml
iot-common = { path = "../iot-common" }
```

Then, from `fake_device/`:

```sh
MOSQUITTO_HOST=127.0.0.1 \
MOSQUITTO_PORT=1883 \
DEVICE_COUNT=1 \
cargo run
```

`MOSQUITTO_USER` and `MOSQUITTO_PASSWORD` are optional environment variables. A local `.env` file is also supported; keep credentials out of version control.

The simulator publishes approximately every 2.5 seconds and retains its messages, unlike the firmware. Its current `1..device_count` loop creates `DEVICE_COUNT - 1` devices when the requested count exceeds one. Account for or correct that behavior when testing fleets.

## Hardware designs

`schematics/` contains KiCad source files and board assets. `enclosure/` contains mechanical and printing assets. These are useful alongside the firmware when adapting the project to a physical device.

Review each design's parts, pin assignments, electrical requirements, and licence before fabrication. Bundled reference hardware and libraries may carry separate terms; the repository is not a single, uniformly licensed hardware design.

## Development notes

- Begin with `bare`, establish provisioning and local HTTP access, then add one hardware module at a time.
- Review dependency versions before updating them. Several manifests use wildcard versions, so resolution is not fully reproducible.
- Treat `devops/` scripts and Kubernetes manifests as historical references. They assume private services, repositories, images, and infrastructure.
- Some tooling paths need repair. For example, `devops/verify_kicad` points to `../kicad-rs`, while the bundled library is at the repository root.
- Keep changes focused and include the board model, enabled features, wiring, and relevant serial logs when reporting firmware issues. Redact credentials from logs.

Useful contributions include reproducible builds, corrected board configurations, driver updates, sensor calibration, clearer hardware documentation, and independent integrations.

## Security

This snapshot has no application authentication on its HTTP administration or actuator routes. The administration page renders stored credentials into its HTML, and configuration handlers print some submitted credentials to serial logs. Provisioning also contains a hard-coded development AP password value; do not rely on it as an access-control boundary.

Run it on an isolated, trusted development network. Add authentication, appropriate transport protection, input validation, credential handling, and actuator safeguards before deploying beyond that environment. The firmware has no demonstrated production security hardening in this release.

## Licensing

A project-wide licence has not yet been supplied in this snapshot. The planned public release needs an explicit licence for the original firmware and a stated licence for the original hardware designs before reuse permissions are clear.

Third-party components retain their own notices and terms, including the licences bundled with `pcd8544/`, `kicad-rs/`, and the reference hardware under `enclosure/`. Those licences do not automatically apply to the rest of Habitat. The proprietary platform is outside the scope of this release.
