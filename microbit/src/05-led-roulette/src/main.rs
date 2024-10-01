#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::display::blocking::Display;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use microbit::board::Board;
use microbit::hal::timer::Timer;
use microbit::hal::prelude::*;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut light_it_all = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    let (mut curr_row, mut curr_col) = (0_i32, 0_i32);
    let (mut prev_row, mut prev_col) = (0_i32, 0_i32);
    let mut dir = (0_i32, 1_i32);
    let (m, n) = (light_it_all.len() as i32, light_it_all[0].len() as i32);
    
    loop {
        light_it_all[prev_row as usize][prev_col as usize] = 0;
        light_it_all[curr_row as usize][curr_col as usize] = 1;

        prev_row = curr_row;
        prev_col = curr_col;

        if curr_row == 0 && curr_col + dir.1 >= n {
            dir = (1, 0);
        } else if curr_row + dir.0 >= m && curr_col == n - 1 {
            dir = (0, -1);
        } else if curr_row == m - 1 && curr_col + dir.1 < 0 {
            dir = (-1, 0);
        } else if curr_row + dir.0 < 0 && curr_col == 0 {
            dir = (0, 1);
        }

        curr_row += dir.0;
        curr_col += dir.1;

        // Show light_it_all for 1000ms
        display.show(&mut timer, light_it_all, 100_u32);
    }
}
