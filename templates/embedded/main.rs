//! {{project_name}} firmware
//! 
//! This is a template for embedded devices created with FerrisUp

#![no_std]
#![no_main]

// Import the panic handler
use panic_halt as _;
use cortex_m_rt::entry;

// This file is a placeholder that will be replaced with a specific implementation
// based on the selected microcontroller target.
// See the target-specific files for detailed implementations.

#[entry]
fn main() -> ! {
    // Device initialization would go here
    
    // Main application loop
    loop {
        // Device main loop
        // - Read sensors
        // - Process data
        // - Control outputs
        // - Handle communication
        
        // This prevents the optimizer from removing the loop
        cortex_m::asm::nop();
    }
}
