#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![allow(clippy::single_match)]

pub mod app;
pub mod light;
pub mod remote;
pub mod show;

extern crate panic_semihosting;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
