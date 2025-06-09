/* ESP32 Memory Layout */
/* This file is mostly a placeholder as ESP32 uses a different linking approach */
/* The actual memory layout is defined by the ESP-IDF and esp32-hal */

MEMORY
{
  /* ESP32 has 4MB of flash and 520KB of SRAM */
  FLASH : ORIGIN = 0x40080000, LENGTH = 4M
  RAM : ORIGIN = 0x3FFC0000, LENGTH = 520K
}
