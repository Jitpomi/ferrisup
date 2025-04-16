# Rust IoT Device Firmware

This template provides a foundation for building IoT device firmware in Rust with connectivity features, sensor integration, and support for various microcontroller platforms.

## Features

- Embedded HAL support for hardware abstraction
- Networking capabilities with embassy-net
- Sensor integration with embedded-sensors
- Platform-specific optimizations for RP2040, ESP32, STM32, and Arduino
- Debugging support with defmt
- Error handling with panic-probe

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd your-project-name
   ```

2. Install the appropriate target for your microcontroller:
   ```bash
   # For RP2040
   rustup target add thumbv6m-none-eabi
   
   # For STM32
   rustup target add thumbv7em-none-eabihf
   
   # For ESP32
   rustup target add xtensa-esp32-none-elf
   
   # For Arduino
   rustup target add avr-unknown-gnu-atmega328
   ```

3. Install additional tools based on your target:
   ```bash
   # For RP2040/STM32
   cargo install probe-run
   
   # For ESP32
   cargo install espflash
   
   # For Arduino
   cargo install ravedude
   ```

4. Build the firmware:
   ```bash
   # Adjust target based on your MCU
   cargo build --target thumbv6m-none-eabi
   ```

5. Flash the firmware:
   ```bash
   # For RP2040/STM32
   cargo run --target thumbv6m-none-eabi
   
   # For ESP32
   espflash flash --monitor target/xtensa-esp32-none-elf/debug/your-project-name
   
   # For Arduino
   cargo run --target avr-unknown-gnu-atmega328
   ```

## Project Structure

- `src/main.rs`: Main application code with IoT functionality
- `Cargo.toml`: Project dependencies and configuration

## Connectivity Features

The template includes support for various connectivity options:

### Wi-Fi (ESP32)
```rust
// Initialize Wi-Fi
let wifi = esp_wifi::initialize(
    esp_wifi::EspWifiInitFor::Wifi,
    timer_group0.timer0,
    rng,
    system.radio_clock_control,
    &clocks,
)?;

// Connect to Wi-Fi
let client_config = Configuration::Client(ClientConfiguration {
    ssid: "your-ssid".into(),
    password: "your-password".into(),
    ..Default::default()
});
```

### Ethernet (STM32)
```rust
// Initialize Ethernet
let eth = ethernet::new(
    dp.ETHERNET_MAC,
    dp.ETHERNET_MTL,
    dp.ETHERNET_DMA,
    &mut rcc.ahb1,
    pins,
);
```

### MQTT
```rust
// Example MQTT connection
let mqtt_client = MqttClient::new(
    "device-id",
    NetworkStack::new(eth),
    core::time::Duration::from_secs(60),
);
```

## Sensor Integration

The template supports various sensors through the embedded-sensors crate:

```rust
// Example temperature sensor
let i2c = I2c::new(dp.I2C1, pins, 400.kHz(), &mut rcc.apb1);
let mut temp_sensor = Temperature::new(i2c);

// Read temperature
let temperature = temp_sensor.read_temperature()?;
```

## Customization

### Adding Custom Sensors

1. Add the appropriate driver crate to your `Cargo.toml`
2. Implement the sensor interface in your code

Example:
```rust
// Add to Cargo.toml
// bme280 = "0.4.4"

// In your code
let bme280 = Bme280::new(i2c, 0x76)?;
let measurements = bme280.measure()?;
```

### Implementing Cloud Connectivity

The template can be extended to connect to various cloud platforms:

- AWS IoT Core
- Azure IoT Hub
- Google Cloud IoT

## Next Steps

- Implement your specific IoT use case
- Add secure storage for credentials
- Implement over-the-air (OTA) updates
- Add power management features
- Set up a CI/CD pipeline for firmware builds

## Resources

- [Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [Embassy Framework Documentation](https://embassy.dev/dev/index.html)
- [ESP-RS Book](https://esp-rs.github.io/book/)
- [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust)
