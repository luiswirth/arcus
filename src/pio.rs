#![no_std]
#![no_main]

use embedded_time::fixed_point::FixedPoint;
use hal::clocks::Clock;
use pico::hal::{self, pac};
use pio;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // init GPIO for PIO
    let sio = hal::sio::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let _pin22: hal::gpio::Pin<_, hal::gpio::FunctionPio0> = pins.gpio22.into_mode();

    // init clocks
    let mut watchdog = hal::watchdog::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let _sysclock = core.SYST;
    let clock_freq = clocks.system_clock.freq().integer();

    let side_set = pio::SideSet::new(false, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    const T1: u8 = 2;
    const T2: u8 = 5;
    const T3: u8 = 3;
    let mut bitloop = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    //let mut wrap_source = assembler.label();
    //assembler.bind(&mut wrap_target);
    assembler.bind(&mut bitloop);
    assembler.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
    assembler.bind(&mut do_one);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut bitloop, T2 - 1, 1);
    assembler.bind(&mut do_zero);
    // Pseudoinstruction: NOP
    assembler.mov_with_delay_and_side_set(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::Y,
        T2 - 1,
        0,
    );

    let program = assembler.assemble(None);

    let pio = hal::pio::PIO::new(pac.PIO0, &mut pac.RESETS);
    let sm = &pio.state_machines()[0];

    let cycles_per_bit = T1 + T2 + T3;
    const FREQ: u32 = 8_000_000;
    let div = clock_freq as f32 / (FREQ as f32 * cycles_per_bit as f32);

    hal::pio::PIOBuilder::default()
        .with_program(&program)
        .buffers(hal::pio::Buffers::OnlyTx)
        .side_set(side_set)
        .side_set_pin_base(22)
        .out_shift_direction(hal::pio::ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(32)
        .clock_divisor(div)
        .build(&pio, sm)
        .unwrap();

    sm.set_enabled(true);

    const NLED: usize = 60;

    let mut data = [0u32; NLED];
    for byte in &mut data {
        *byte = u32::MAX;
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

    loop {
        for word in data {
            sm.push(word);
        }
        cortex_m::asm::delay(50_000);
        cortex_m::asm::nop();
    }
}
