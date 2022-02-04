let mut flash = device.FLASH.constrain();
let mut rcc = device.RCC.constrain();

let clocks = rcc
    .cfgr
    .use_hse(8.mhz())
    .sysclk(48.mhz())
    .pclk1(24.mhz())
    .freeze(&mut flash.acr);
