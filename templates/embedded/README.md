# {{project_name}}

A Rust embedded firmware project for {{mcu_target}} microcontrollers created with FerrisUp.

## Setup

This project requires the following tools:

1. Rust and Cargo (install via [rustup](https://rustup.rs/))
2. The appropriate target for your microcontroller:
   - For RP2040: `rustup target add thumbv6m-none-eabi`
   - For STM32: `rustup target add thumbv7em-none-eabihf`
   - For ESP32: `rustup target add xtensa-esp32-none-elf`
   - For Arduino: `rustup target add avr-unknown-gnu-atmega328`

3. Additional tools based on your target:
   - For RP2040: [probe-run](https://github.com/knurling-rs/probe-run) (`cargo install probe-run`)
   - For ESP32: [espflash](https://github.com/esp-rs/espflash) (`cargo install espflash`)
   - For Arduino: [ravedude](https://github.com/Rahix/avr-hal/tree/main/ravedude) (`cargo install ravedude`)

## Building

```bash
# For RP2040
cargo build --target thumbv6m-none-eabi

# For STM32
cargo build --target thumbv7em-none-eabihf

# For ESP32
cargo build --target xtensa-esp32-none-elf

# For Arduino
cargo build --target avr-unknown-gnu-atmega328
```

## Flashing

```bash
# For RP2040
cargo run --target thumbv6m-none-eabi

# For STM32 (using probe-run)
cargo run --target thumbv7em-none-eabihf

# For ESP32
espflash flash --monitor target/xtensa-esp32-none-elf/debug/{{project_name}}

# For Arduino
cargo run --target avr-unknown-gnu-atmega328
```

## Project Structure

- `src/main.rs`: Main application code
- `memory.x`: Memory layout for the microcontroller
- `.cargo/config.toml`: Cargo configuration for embedded targets

## Resources

- [Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [Embedded Rust Discovery Book](https://docs.rust-embedded.org/discovery/)
- [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
