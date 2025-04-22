//! ESP32 firmware
//! 
//! This is a template for ESP32 devices created with FerrisUp

#![no_std]
#![no_main]

use esp32_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
use esp_println::println;

#[entry]
fn main() -> ! {
    // Initialize the ESP32 device
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    
    // Get the RTC and timer group
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    
    // Print a welcome message
    println!("Hello from ESP32!");
    println!("ESP32 Firmware is running!");
    
    // Main application loop
    loop {
        // Your application code here
        println!("Tick...");
        
        // Delay for a second
        timer0.delay(1.secs());
    }
}
