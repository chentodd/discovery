#![no_main]
#![no_std]

use cortex_m_rt::entry;
use core::fmt::Write;
use heapless::Vec;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

use microbit::hal::prelude::*;

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
};

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

use lsm303agr::{
    AccelOutputDataRate, MagOutputDataRate, Lsm303agr
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let mut i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    
    loop {
        let byte: u8 = nb::block!(serial.read()).unwrap();
        if byte != b'\n' {
            buffer.push(byte).unwrap();
        }

        if byte == b'\r' {
            buffer.pop();
            let command: &str = core::str::from_utf8(&buffer).unwrap();

            if command == "magnetometer" {
                let data = sensor.mag_data().unwrap();
                write!(serial, "Magnetometer: x {} y {} z {}\r\n", data.x, data.y, data.z);
            } else if command == "accelerometer" {
                let data = sensor.accel_data().unwrap();
                write!(serial, "Acceleration: x {} y {} z {}\r\n", data.x, data.y, data.z);
            }
            buffer.clear();
        }
        nb::block!(serial.flush()).unwrap();
    }
}
