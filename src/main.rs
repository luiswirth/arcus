#![no_std]
#![no_main]

mod debug;

use core::fmt::Write;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_time::fixed_point::FixedPoint;
use pico_explorer::{
    hal::{
        self,
        adc::Adc,
        clocks::ClockSource,
        sio::Sio,
        uart::UartPeripheral,
        watchdog::Watchdog,
    },
    pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

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
    let mut delay = Delay::new(cp.SYST, clocks.system_clock.get_freq().integer());
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
    //let _pin0: hal::gpio::Pin<_, hal::gpio::FunctionPio0> = pins.gpio0.into_push_pull_output().into_mode();
    let _pin2: hal::gpio::Pin<_, hal::gpio::FunctionPio0> = pins.gpio2.into_mode();

    let led = pins.led.into_push_pull_output();

    debug::init_debug(led, explorer.screen);
    let mut string = debug::ArrayString::new();

    let side_set = pio::SideSet::new(true, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    // configure pin as output
    // TODO: is this really necessary for side_set?
    assembler.set(pio::SetDestination::PINDIRS, 1);

    let mut wrap_target = assembler.label();
    let mut bitloop = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    let mut wrap_source = assembler.label();

    const T0: u8 = 2;
    const T1: u8 = 2;
    const T2: u8 = 2;
    const T3: u8 = 4;

    assembler.bind(&mut wrap_target);
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
    assembler.bind(&mut wrap_source);

    let program = assembler.assemble(Some((wrap_source, wrap_target)));

    let pio = hal::pio::PIO::new(p.PIO0, &mut p.RESETS);
    let sm = &pio.state_machines()[0];

    let cycles_per_bit = T0 + T1 + T2 + T3;
    const TIME: f32 = 150e-9;
    const FREQ: f32 = 1.0 / TIME;
    let clock_freq = clocks.system_clock.get_freq().0;
    let div = clock_freq as f32 / (FREQ as f32 * cycles_per_bit as f32);

    writeln!(string, "target_freq: {}", FREQ).unwrap();
    writeln!(string, "clock_freq: {}", clock_freq).unwrap();
    writeln!(string, "div: {}", div).unwrap();

    let builder = hal::pio::PIOBuilder::default()
        .with_program(&program)
        .buffers(hal::pio::Buffers::OnlyTx)
        .out_shift_direction(hal::pio::ShiftDirection::Right)
        .autopull(true)
        .pull_threshold(32)
        .clock_divisor(div)
        .set_pins(2, 1)
        .side_set(side_set)
        .side_set_pin_base(2);

    builder.build(&pio, sm).unwrap();

    sm.set_enabled(true);

    const NLED: usize = 60;

    let string = debug::breakup(string);
    debug::sprint(&string);

    let uart = UartPeripheral::<_, _>::enable(
        p.UART0,
        &mut p.RESETS,
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
    )
    .unwrap();

    let _tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();

    uart.write_full_blocking(b"UART Test!!!\r\n");
    hprintln!("semihosting test!!").unwrap();

    loop {
        sm.push(0xFF_FF_FF_FF);
        sm.push(0xFF_00_00_00);
        sm.push(0x00_FF_00_00);
        sm.push(0x00_00_FF_00);
        sm.push(0x00_00_00_FF);
        delay.delay_ms(1000);
        //cortex_m::asm::nop();
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
