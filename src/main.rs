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

    let side_set = pio::SideSet::new(false, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    //      low      high
    // 0:   0.85us + 0.40us
    // 1:   0.45us + 0.8us
    //
    // T = 1 is 125ns = 0.125us
    //
    // REAL
    //      
    // 0: l 0.375us + h 0.25us + l 0.625 == 0.25 h + 1.0 l
    // 1: l 0.375us + h 0.25us + h 0.625 == 0.875 h + 0.375 l
    const T1: u8 = 2;
    const T2: u8 = 5;
    const T3: u8 = 3;
    let mut bitloop = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    assembler.bind(&mut bitloop);
    assembler.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0); // 0.375us
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1); // 0.25us
    assembler.bind(&mut do_one);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut bitloop, T2 - 1, 1); // 0.625us
    assembler.bind(&mut do_zero);
    // Pseudoinstruction: NOP
    assembler.mov_with_delay_and_side_set( // 0.625us
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::Y,
        T2 - 1,
        0,
    );

    writeln!(string, "{:?}", assembler).unwrap();

    let program = assembler.assemble(None);
    writeln!(string, "{:?}", program).unwrap();

    let pio = hal::pio::PIO::new(p.PIO0, &mut p.RESETS);
    writeln!(string, "{:?}", pio).unwrap();
    let sm = &pio.state_machines()[0];

    let cycles_per_bit = T1 + T2 + T3;
    const TIME: f32 = 300e-9;
    const FREQ: f32 = 1.0 / TIME;
    let div = clock_freq as f32 / (FREQ as f32 * cycles_per_bit as f32);

    let builder = hal::pio::PIOBuilder::default()
        .with_program(&program)
        .buffers(hal::pio::Buffers::OnlyTx)
        .side_set(side_set)
        .side_set_pin_base(0)
        .out_shift_direction(hal::pio::ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(32)
        .clock_divisor(div);

    writeln!(string, "{:?}", builder).unwrap();
    builder.build(&pio, sm).unwrap();

    sm.set_enabled(true);

    const NLED: usize = 2;
    //let data = [0xFFFFFF, 0];
    let data = [0, 0xFFFFFF];

    //let data = [0xFF0000; NLED];
    for (i, word) in data.iter().enumerate() {
        if !sm.write_tx(*word) {
            panic!("not written! i={}, w={}", i, word);
        }
    }

    let string = debug::breakup(string);
    debug::sprint(&string);

    loop {
        cortex_m::asm::nop();
    }
}

#[allow(dead_code)]
fn color([r, g, b, w]: [u8; 4]) -> u32 {
    let mut grbw = 0u32;
    grbw |= (g as u32) << 24;
    grbw |= (r as u32) << 16;
    grbw |= (b as u32) << 8;
    grbw |= w as u32;
    grbw
}
