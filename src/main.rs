#![no_std]
#![no_main]

mod debug;

use debug::SCREEN_SIZE2;

use cortex_m_rt::entry;
use embedded_time::fixed_point::FixedPoint;
use pico_explorer::{
    hal::{self, adc::Adc, clocks::ClockSource, sio::Sio, watchdog::Watchdog},
    pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

use crate::debug::SCREEN_SIZE;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // Enable watchdog and clocks
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        p.XOSC,
        p.CLOCKS,
        p.PLL_SYS,
        p.PLL_USB,
        &mut p.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(cp.SYST, clocks.system_clock.get_freq().integer());

    let adc = Adc::new(p.ADC, &mut p.RESETS);
    let sio = Sio::new(p.SIO);

    let (explorer, pins) = PicoExplorer::new(
        p.IO_BANK0,
        p.PADS_BANK0,
        sio.gpio_bank0,
        p.SPI0,
        adc,
        &mut p.RESETS,
        &mut delay,
    );

    let led = pins.led.into_push_pull_output();

    debug::init_debug(led, explorer.screen);

    let mut text = debug::ArrayString::new();
    text.push_str(include_str!("../res/faust.txt"));
    let text = debug::breakup(text);

    let pixels = 0..SCREEN_SIZE2 as u16;
    debug::screen_mut()
        .set_pixels(0, 0, SCREEN_SIZE - 1, SCREEN_SIZE - 1, pixels)
        .unwrap();

    debug::draw_text(&text);

    loop {
        cortex_m::asm::nop();
    }
}
