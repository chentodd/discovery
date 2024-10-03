#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    pac::twi0::frequency::FREQUENCY_A,
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{
    AccelScale, AccelOutputDataRate, Lsm303agr,
};

use microbit::hal::timer::Timer;
use microbit::hal::prelude::*;
use nb::Error;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut countdown = Timer::new(board.TIMER0);
    let mut delay = Timer::new(board.TIMER1);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_accel_scale(AccelScale::G16).unwrap();

    const THRESHOLD: f32 = 0.4;
    let mut start_measuring: bool = false;
    let mut max_acc_x: f32 = 0.0;
    loop {
        while !sensor.accel_status().unwrap().xyz_new_data {}
        
        let curr_acc_x = sensor.accel_data().unwrap().x as f32 / 1000.0;
        if !start_measuring && curr_acc_x > THRESHOLD {
            rprintln!("START measuring...");

            start_measuring = true;
            max_acc_x = curr_acc_x;

            countdown.start(1_000_000_u32);
        } else if start_measuring {
            match countdown.wait() {
                // countdown isn't done yet
                Err(Error::WouldBlock) => {
                    if curr_acc_x > max_acc_x {
                        max_acc_x = curr_acc_x;
                    }
                },
                // Countdown is done
                Ok(_) => {
                    // Report max value
                    rprintln!("Max acceleration: {}g", max_acc_x);

                    // Reset
                    max_acc_x = 0.;
                    start_measuring = false;
                },
                // Since the nrf52 and nrf51 HAL have Void as an error type
                // this path cannot occur, as Void is an empty type
                Err(Error::Other(_)) => {
                    unreachable!()
                }
            }
        }

        delay.delay_ms(20_u8);
    }
}
