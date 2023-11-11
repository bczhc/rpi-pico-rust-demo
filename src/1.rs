#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate alloc;

use core::fmt::Write;

use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, Network, PrivateKey};
// use rp_pico::entry;
use cortex_m_rt::entry;
use digest::Digest;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
#[allow(unused_imports)]
use panic_halt as _;
use rand::{Error, RngCore};
use rp_pico::hal;
use rp_pico::pac::ROSC;

use pico_rs::{clocks, peripherals, pins, set_up_allocator, timer, uart};

#[entry]
fn main() -> ! {
    set_up_allocator();

    let mut pac = peripherals();
    let pins = pins!(pac);
    let clocks = clocks!(pac);
    let mut timer = timer!(&clocks, pac);
    let mut uart = uart!(pins, pac, clocks, 115200);
    let mut led = pins.led.into_push_pull_output();

    macro_rules! uart_println {
        ($($arg:tt)*) => {
            writeln!(&mut uart, $($arg)*).unwrap();
        };
    }

    macro_rules! led_hint {
        () => {
            led.set_high().unwrap();
            timer.delay_ms(10);
            led.set_low().unwrap();
        };
    }

    let k1 = Secp256k1::new();
    loop {
        led_hint!();
        let secret = random_secret();
        let private_key =
            PrivateKey::new(SecretKey::from_slice(&secret).unwrap(), Network::Bitcoin);
        let public_key = private_key.public_key(&k1);
        let address = Address::p2wpkh(&public_key, Network::Bitcoin).unwrap();
        uart_println!("{} {}", private_key, address);
    }
}

fn random_secret() -> [u8; 32] {
    let mut ec = [0_u8; 32];
    RoscRng.fill_bytes(&mut ec);
    ec
}

struct RoscRng;

impl RoscRng {
    #[inline]
    fn random_byte() -> u8 {
        let mut b = 0_u8;
        for i in 0..8 {
            let bits = unsafe { (*ROSC::PTR).randombit.read().bits() } as u8;
            b |= (bits & 0b1) << i;
        }
        b
    }
}

impl RngCore for RoscRng {
    fn next_u32(&mut self) -> u32 {
        let mut u = [0_u8; 4];
        self.fill_bytes(&mut u);
        u32::from_le_bytes(u)
    }

    fn next_u64(&mut self) -> u64 {
        let mut u = [0_u8; 8];
        self.fill_bytes(&mut u);
        u64::from_le_bytes(u)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for x in dest.iter_mut() {
            *x = Self::random_byte();
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
