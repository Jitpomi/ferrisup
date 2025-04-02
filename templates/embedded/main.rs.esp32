//! ESP32 firmware
//! 
//! This is a template for ESP32 microcontrollers created with FerrisUp

#![no_std]
#![no_main]

// Import the panic handler
use esp_backtrace as _;

// ESP32-specific imports
use esp32_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay,
    Rtc,
};

use esp_println::println;

#[entry]
fn main() -> ! {
    // Take peripherals
    let peripherals = Peripherals::take();
    
    // Initialize the clock control
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    
    // Initialize the RTC
    let rtc = Rtc::new(peripherals.RTC_CNTL);
    
    // Initialize timer group 0
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    
    // Set up the GPIO pins
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    
    // Configure GPIO2 as an output (built-in LED on most ESP32 dev boards)
    let mut led = io.pins.gpio2.into_push_pull_output();
    
    // Create a delay provider
    let mut delay = Delay::new(&clocks);
    
    // Print a welcome message
    println!("ESP32 Firmware Starting...");
    
    // Main application loop
    loop {
        // Toggle the LED
        led.set_high().unwrap();
        delay.delay_ms(500u32);
        led.set_low().unwrap();
        delay.delay_ms(500u32);
        
        // Print a message
        println!("LED toggled");
        
        // In a real application, you would:
        // - Read sensors
        // - Process data
        // - Control outputs
        // - Handle WiFi/BLE communication
    }
}
