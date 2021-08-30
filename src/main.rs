#![no_std]
#![no_main]

mod debug;

use core::fmt::Write;
use debug::SCREEN_SIZE2;

use cortex_m_rt::entry;
use embedded_time::fixed_point::FixedPoint;
use pico_explorer::{
    hal::{
        self,
        adc::Adc,
        clocks::{Clock, ClockSource},
        sio::Sio,
        watchdog::Watchdog,
    },
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
    let _pin0: hal::gpio::Pin<_, hal::gpio::FunctionPio0> = pins.gpio0.into_mode();

    let led = pins.led.into_push_pull_output();

    debug::init_debug(led, explorer.screen);
    let mut string = debug::ArrayString::new();

    let clock_freq = clocks.system_clock.freq().integer();

    // maybe (false, 1, true)
    let side_set = pio::SideSet::new(true, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    // configure pin as output
    assembler.set(pio::SetDestination::PINDIRS, 1);

    const T0: u8 = 2;
    const T1: u8 = 2;
    const T2: u8 = 2;
    const T3: u8 = 4;

    let mut bitloop = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    assembler.bind(&mut bitloop);
    assembler.out_with_delay_and_side_set(pio::OutDestination::X, 1, T0 - 1, 0);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
    assembler.bind(&mut do_one);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut bitloop, T2 - 1, 1);
    assembler.bind(&mut do_zero);
    // Pseudoinstruction: NOP
    assembler.mov_with_delay_and_side_set(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::Y,
        T3 - 1,
        0,
    );

    let program = assembler.assemble(None);

    let pio = hal::pio::PIO::new(p.PIO0, &mut p.RESETS);
    let sm = &pio.state_machines()[0];

    let cycles_per_bit = T1 + T2 + T3;
    const TIME: f32 = 150e-9;
    const FREQ: f32 = 1.0 / TIME;
    let div = clock_freq as f32 / (FREQ as f32 * cycles_per_bit as f32);

    let builder = hal::pio::PIOBuilder::default()
        .with_program(&program)
        .buffers(hal::pio::Buffers::OnlyTx)
        .set_pins(0, 1)
        .side_set(side_set)
        .side_set_pin_base(0)
        .out_shift_direction(hal::pio::ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(32)
        .clock_divisor(div);

    builder.build(&pio, sm).unwrap();

    sm.set_enabled(true);

    const NLED: usize = 60;

    for _ in 0..NLED {
        sm.push(color([255, 255, 255, 255]));
    }

    let string = debug::breakup(string);
    debug::sprint(&string);

    loop {
        cortex_m::asm::nop();
    }
}

fn color([r, g, b, w]: [u8; 4]) -> u32 {
    let mut grbw = 0u32;
    grbw |= (g as u32) << 24;
    grbw |= (r as u32) << 16;
    grbw |= (b as u32) << 8;
    grbw |= w as u32;
    grbw
}
