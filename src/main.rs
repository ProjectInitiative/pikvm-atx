//! # Multicore FIFO + GPIO 'Blinky' Example
//!
//! This application demonstrates FIFO communication between the CPU cores on the RP2040.
//! Core 0 will calculate and send a delay value to Core 1, which will then wait that long
//! before toggling the LED.
//! Core 0 will wait for Core 1 to complete this task and send an acknowledgement value.
//!
//! It may need to be adapted to your particular board layout and/or pin assignment.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

use hal::clocks::Clock;
use hal::multicore::{Multicore, Stack};
use hal::sio::Sio;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp2040_hal as hal;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use hal::pac;

// Some traits we need
use embedded_hal::digital::v2::OutputPin;
// use embedded_hal::digital::v2::ToggleableOutputPin;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Value to indicate that Core 1 has completed its task
const CORE1_TASK_COMPLETE: u32 = 0xEE;

/// Values to indicate ATX actions
const RESET_DELAY: u32 = 100;
const POWER_DELAY_SHORT: u32 = 100;
const POWER_DELAY_LONG: u32 = 5000;

const SERV1_RESET: u32 = u32::from_be_bytes([b'S', b'1', b'R', b'S']);
const SERV1_POWER_SHORT: u32 = u32::from_be_bytes([b'S', b'1', b'P', b'S']);
const SERV1_POWER_LONG: u32 = u32::from_be_bytes([b'S', b'1', b'P', b'L']);

const SERV2_RESET: u32 = u32::from_be_bytes([b'S', b'2', b'R', b'S']);
const SERV2_POWER_SHORT: u32 = u32::from_be_bytes([b'S', b'2', b'P', b'S']);
const SERV2_POWER_LONG: u32 = u32::from_be_bytes([b'S', b'2', b'P', b'L']);

const SERV3_RESET: u32 = u32::from_be_bytes([b'S', b'3', b'R', b'S']);
const SERV3_POWER_SHORT: u32 = u32::from_be_bytes([b'S', b'3', b'P', b'S']);
const SERV3_POWER_LONG: u32 = u32::from_be_bytes([b'S', b'3', b'P', b'L']);

const SERV4_RESET: u32 = u32::from_be_bytes([b'S', b'4', b'R', b'S']);
const SERV4_POWER_SHORT: u32 = u32::from_be_bytes([b'S', b'4', b'P', b'S']);
const SERV4_POWER_LONG: u32 = u32::from_be_bytes([b'S', b'4', b'P', b'L']);

/// Stack for core 1
///
/// Core 0 gets its stack via the normal route - any memory not used by static values is
/// reserved for stack and initialised by cortex-m-rt.
/// To get the same for Core 1, we would need to compile everything seperately and
/// modify the linker file for both programs, and that's quite annoying.
/// So instead, core1.spawn takes a [usize] which gets used for the stack.
/// NOTE: We use the `Stack` struct here to ensure that it has 32-byte alignment, which allows
/// the stack guard to take up the least amount of usable RAM.
static mut CORE1_STACK: Stack<4096> = Stack::new();

fn core1_task(sys_freq: u32) -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };

    let mut sio = Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // define all of the server input switches that are required
    let mut serv1_reset_pin = pins.gpio2.into_push_pull_output();
    let mut serv1_power_pin = pins.gpio3.into_push_pull_output();
    let mut serv2_reset_pin = pins.gpio4.into_push_pull_output();
    let mut serv2_power_pin = pins.gpio5.into_push_pull_output();
    let mut serv3_reset_pin = pins.gpio6.into_push_pull_output();
    let mut serv3_power_pin = pins.gpio7.into_push_pull_output();
    let mut serv4_reset_pin = pins.gpio8.into_push_pull_output();
    let mut serv4_power_pin = pins.gpio9.into_push_pull_output();

    let mut led_pin = pins.gpio25.into_push_pull_output();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq);
    loop {
        //blink board LED anytime a message comes in
        led_pin.set_low().unwrap();
        let input = sio.fifo.read();

        if let Some(input) = input {
            led_pin.set_high().unwrap();
            match input {
                SERV1_RESET => {
                    serv1_reset_pin.set_high().unwrap();
                    delay.delay_ms(RESET_DELAY);
                    serv1_reset_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV1_POWER_SHORT => {
                    serv1_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_SHORT);
                    serv1_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV1_POWER_LONG => {
                    serv1_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_LONG);
                    serv1_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }

                SERV2_RESET => {
                    serv2_reset_pin.set_high().unwrap();
                    delay.delay_ms(RESET_DELAY);
                    serv2_reset_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV2_POWER_SHORT => {
                    serv2_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_SHORT);
                    serv2_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV2_POWER_LONG => {
                    serv2_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_LONG);
                    serv2_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }

                SERV3_RESET => {
                    serv3_reset_pin.set_high().unwrap();
                    delay.delay_ms(RESET_DELAY);
                    serv3_reset_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV3_POWER_SHORT => {
                    serv3_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_SHORT);
                    serv3_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV3_POWER_LONG => {
                    serv3_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_LONG);
                    serv3_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }

                SERV4_RESET => {
                    serv4_reset_pin.set_high().unwrap();
                    delay.delay_ms(RESET_DELAY);
                    serv4_reset_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV4_POWER_SHORT => {
                    serv4_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_SHORT);
                    serv4_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                SERV4_POWER_LONG => {
                    serv4_power_pin.set_high().unwrap();
                    delay.delay_ms(POWER_DELAY_LONG);
                    serv4_power_pin.set_low().unwrap();
                    sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
                }
                _ => sio.fifo.write_blocking(CORE1_TASK_COMPLETE),
            }
        };
    }
}

#[macro_use]
extern crate arrayref;

fn chop(buf: &[u8]) -> &[u8; 4] {
    array_ref!(buf, 0, 4)
}

/// Entry point to our bare-metal application.
///
/// The `#[rp2040_hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the RP2040 peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[rp2040_hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::watchdog::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sys_freq = clocks.system_clock.freq().to_Hz();

    // The single-cycle I/O block controls our GPIO pins
    let mut sio = hal::sio::Sio::new(pac.SIO);

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        core1_task(sys_freq)
    });

    let mut delay = cortex_m::delay::Delay::new(_core.SYST, sys_freq);

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0232, 0x0232))
        .manufacturer("pikvm-atx")
        .product("atx-serial-adapter")
        .serial_number("232")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    // let mut said_hello = false;
    loop {
        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    // Convert to upper case
                    buf.iter_mut().take(count).for_each(|b| {
                        b.make_ascii_uppercase();
                    });
                    if count == 4 {
                        let new_buf: [u8; 4] = chop(&buf).clone();
                        // Send the new delay time to Core 1. We convert it
                        sio.fifo.write(u32::from_be_bytes(new_buf));

                        // Sleep until Core 1 sends a message to tell us it is done
                        let ack = sio.fifo.read_blocking();
                        if ack != CORE1_TASK_COMPLETE {
                            // In a real application you might want to handle the case
                            // where the CPU sent the wrong message - we're going to
                            // ignore it here.
                        }
                    }
                    // Send back to the host
                    let mut wr_ptr = &buf[..count];
                    while !wr_ptr.is_empty() {
                        match serial.write(wr_ptr) {
                            Ok(len) => wr_ptr = &wr_ptr[len..],
                            // On error, just drop unwritten data.
                            // One possible error is Err(WouldBlock), meaning the USB
                            // write buffer is full.
                            Err(_) => break,
                        };
                    }
                }
            }
        }
    }
}

// End of file
