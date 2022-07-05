use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use rp_pico::{
  hal::{gpio, uart},
  pac,
};

type UartPin0 = gpio::Pin<gpio::bank0::Gpio0, gpio::FunctionUart>;
type UartPin1 = gpio::Pin<gpio::bank0::Gpio1, gpio::FunctionUart>;
type UartPins = (UartPin0, UartPin1);
type UartPeripheral = uart::UartPeripheral<uart::Enabled, rp_pico::pac::UART0, UartPins>;

pub static UART_PERIPHERAL: Mutex<RefCell<Option<UartPeripheral>>> = Mutex::new(RefCell::new(None));

pub fn init_uart(
  peripheral: rp_pico::pac::UART0,
  resets: &mut pac::RESETS,
  pin0: UartPin0,
  pin1: UartPin1,
  frequency: embedded_time::rate::Hertz,
) {
  let uart_pins = (pin0, pin1);
  let mut peripheral: UartPeripheral = uart::UartPeripheral::new(peripheral, uart_pins, resets)
    .enable(uart::common_configs::_115200_8_N_1, frequency)
    .unwrap();
  peripheral.enable_rx_interrupt();

  cortex_m::interrupt::free(|cs| {
    UART_PERIPHERAL.borrow(cs).replace(Some(peripheral));
  });
}

#[macro_export]
macro_rules! uprint {
  ($($arg:tt)*) => {
    cortex_m::interrupt::free(|cs| {
      use core::fmt::Write;
      let uart = UART_PERIPHERAL
        .borrow(cs)
        .borrow_mut()
        .as_mut()
        .expect("uart not initialized");
      write!(uart, $($arg)*)
    });
  };
}

#[macro_export]
macro_rules! uprintln {
  ($($arg:tt)*) => {
    cortex_m::interrupt::free(|cs| {
      use core::fmt::Write;
      let mut uart = $crate::util::uart::UART_PERIPHERAL
        .borrow(cs)
        .borrow_mut();
      let uart = uart
        .as_mut()
        .expect("uart not initialized");
      writeln!(uart, $($arg)*).unwrap();
    })
  };
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
  uprintln!("{}", panic_info);
  loop {}
}
