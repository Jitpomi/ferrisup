//! Raspberry Pi Pico (RP2040) firmware
//! 
//! This is a template for RP2040 microcontroller created with FerrisUp

#![no_std]
#![no_main]

// Import the panic handler
use panic_halt as _;

// RP2040-specific imports
use rp2040_hal as hal;
use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use cortex_m::delay::Delay;
use cortex_m_rt::entry;

// USB Device support
use hal::usb::UsbBus;
use usb_device::{class_prelude::*, prelude::*};

// The linker will place this boot block at the start of our program
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    
    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    
    // Configure the clocks
    let clocks = init_clocks_and_plls(
        12_000_000,  // Crystal frequency
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    
    // Set up the USB Communications Class Device driver
    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    
    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("FerrisUp")
        .product("RP2040 Example")
        .serial_number("TEST")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    
    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(pac.SIO);
    
    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    // Configure GPIO25 as an output (Pico's built-in LED)
    let mut led_pin = pins.gpio25.into_push_pull_output();
    
    // Create a delay abstraction based on the cortex-m systick
    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Main application loop
    loop {
        // Toggle the LED
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
        
        // Poll the USB device
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    // Echo back in upper case
                    for c in buf[0..count].iter_mut() {
                        if 0x61 <= *c && *c <= 0x7a {
                            *c &= !0x20;
                        }
                    }
                    
                    // Write data back to the USB serial port
                    let _ = serial.write(&buf[0..count]);
                }
                _ => {}
            }
        }
    }
}
