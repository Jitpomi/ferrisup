//! Arduino firmware
//! 
//! This is a template for Arduino microcontrollers created with FerrisUp

#![no_std]
#![no_main]

// Import the panic handler
use panic_halt as _;

// Arduino-specific imports
use arduino_hal::{
    prelude::*,
    delay::Delay,
    hal::port::{PD5, PB5},
    port::{
        mode::{Output, Input},
        Pin,
    },
};

// Define the entry point for Arduino
#[arduino_hal::entry]
fn main() -> ! {
    // Get access to the Arduino peripherals
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    
    // Create a delay instance
    let mut delay = arduino_hal::Delay::new();
    
    // Configure the built-in LED (pin 13 on most Arduino boards)
    let mut led = pins.d13.into_output();
    
    // Configure a button on pin 2
    let button = pins.d2.into_pull_up_input();
    
    // Initialize serial communication
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // Print a welcome message
    ufmt::uwriteln!(&mut serial, "Arduino Firmware Starting...").void_unwrap();
    
    // Main application loop
    loop {
        // Check if the button is pressed
        if button.is_low() {
            // Button is pressed, turn on the LED
            led.set_high();
            ufmt::uwriteln!(&mut serial, "Button pressed - LED ON").void_unwrap();
        } else {
            // Button is not pressed, blink the LED
            led.toggle();
            ufmt::uwriteln!(&mut serial, "LED toggled").void_unwrap();
        }
        
        // Delay for a short time
        delay.delay_ms(500);
        
        // In a real application, you would:
        // - Read sensors
        // - Process data
        // - Control outputs
        // - Handle communication
    }
}
