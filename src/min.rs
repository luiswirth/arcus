#![no_std]
#![no_main]

mod debug;

use cortex_m_rt::entry;
use hal::{clocks::init_clocks_and_plls, pac, watchdog::Watchdog};
use pico_explorer::{XOSC_CRYSTAL_FREQ, hal as hal};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let _clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::sio::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    // Configure led pin for PIO0 use
    let _: hal::gpio::Pin<_, hal::gpio::FunctionPio0> = pins.gpio25.into_mode();

    let mut a = pio::Assembler::new();

    let mut toggle_loop = a.label();
    let mut delay = a.label();

    // configure pin as output
    a.set(pio::SetDestination::PINDIRS, 1);

    a.bind(&mut toggle_loop);
    a.set(pio::SetDestination::X, 8);
    a.bind(&mut delay);
    // NOP with all 5 bits of delay
    a.mov_with_delay(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::Y,
        0b11111,
    );
    // continue to NOP until X scratch register is 0
    a.jmp(pio::JmpCondition::XDecNonZero, &mut delay);
    // toggle the output pin
    a.mov(
        pio::MovDestination::PINS,
        pio::MovOperation::Invert,
        pio::MovSource::PINS,
    );
    a.jmp(pio::JmpCondition::Always, &mut toggle_loop);

    let program = a.assemble(None);

    let pio = hal::pio::PIO::new(pac.PIO0, &mut pac.RESETS);
    let sm = &pio.state_machines()[0];

    hal::pio::PIOBuilder::default()
        .with_program(&program)
        .buffers(hal::pio::Buffers::RxTx)
        .out_pins(25, 1)
        .in_pin_base(25)
        .set_pins(25, 1)
        .clock_divisor(0xffff as f32)
        .build(&pio, sm)
        .unwrap();

    sm.set_enabled(true);

    loop {}
}
