//! STM32 firmware
//! 
//! This is a template for STM32 microcontrollers created with FerrisUp

#![no_std]
#![no_main]

// Import the panic handler
use panic_halt as _;

// STM32-specific imports
use stm32f4xx_hal as hal;
use hal::{
    pac,
    prelude::*,
    timer::Timer,
    gpio::{Output, PushPull},
};

use cortex_m::delay::Delay;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies
    // in `clocks`
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks.sysclk().0);

    // Setup the LED pin as a push-pull output.
    // On the STM32F411 Discovery board, the user LED is connected to pin PA5.
    let gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    // Configure TIM2 as a periodic timer
    let mut timer = Timer::tim2(dp.TIM2, 1.Hz(), clocks);

    // Main application loop
    loop {
        // Toggle the LED
        led.set_high();
        delay.delay_ms(500_u32);
        led.set_low();
        delay.delay_ms(500_u32);
        
        // Wait for the timer event
        timer.wait().unwrap();
        
        // In a real application, you would:
        // - Read sensors
        // - Process data
        // - Control outputs
        // - Handle communication
    }
}
