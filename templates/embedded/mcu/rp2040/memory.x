/* RP2040 Memory Layout */
MEMORY
{
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}

/* The boot loader */
EXTERN(BOOT2_FIRMWARE)

/* The second stage bootloader */
EXTERN(BOOT_LOADER)

EXTERN(__DEFMT_MARKER)

/* Required entry point */
ENTRY(reset_handler)
