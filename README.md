# arcus

## Run
Running requires the rp2040 fork of openocd
and arm-none-eabi-gdb, arm-none-eabi-gcc, arm-none-eabi-newlib.

```
openocd
cargo run
```

## UART

requires minicom

```
minicom -b 115200 -o -D /dev/ttyACM0
```
