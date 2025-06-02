# {{project_name}}

A Rust embedded firmware project for {{mcu_target}} microcontrollers created with FerrisUp.

## Setup

This project requires the following tools:

1. Rust and Cargo (install via [rustup](https://rustup.rs/))
2. The appropriate target for your microcontroller:
   {{#if (eq mcu_target "rp2040")}}
   - `rustup target add thumbv6m-none-eabi`
   - [probe-run](https://github.com/knurling-rs/probe-run) (`cargo install probe-run`)
   {{/if}}
   {{#if (eq mcu_target "stm32")}}
   - `rustup target add thumbv7em-none-eabihf`
   - [probe-run](https://github.com/knurling-rs/probe-run) (`cargo install probe-run`)
   {{/if}}
   {{#if (eq mcu_target "esp32")}}
   - `rustup target add xtensa-esp32-none-elf`
   - [espflash](https://github.com/esp-rs/espflash) (`cargo install espflash`)
   {{/if}}
   {{#if (eq mcu_target "arduino")}}
   - `rustup target add avr-unknown-gnu-atmega328`
   - [ravedude](https://github.com/Rahix/avr-hal/tree/main/ravedude) (`cargo install ravedude`)
   {{/if}}

## Building

```bash
{{#if (eq mcu_target "rp2040")}}
cargo build --target thumbv6m-none-eabi
{{/if}}
{{#if (eq mcu_target "stm32")}}
cargo build --target thumbv7em-none-eabihf
{{/if}}
{{#if (eq mcu_target "esp32")}}
cargo build --target xtensa-esp32-none-elf
{{/if}}
{{#if (eq mcu_target "arduino")}}
cargo build --target avr-unknown-gnu-atmega328
{{/if}}
```

## Flashing

```bash
{{#if (eq mcu_target "rp2040")}}
cargo run --target thumbv6m-none-eabi
{{/if}}
{{#if (eq mcu_target "stm32")}}
cargo run --target thumbv7em-none-eabihf
{{/if}}
{{#if (eq mcu_target "esp32")}}
espflash flash --monitor target/xtensa-esp32-none-elf/debug/{{project_name}}
{{/if}}
{{#if (eq mcu_target "arduino")}}
cargo run --target avr-unknown-gnu-atmega328
{{/if}}
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
