//! STM32F303RE example using stm32-metapac

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::{entry};
use stm32_metapac as device;
use device::{gpio::vals::Moder, flash::vals::Latency};

#[entry]
fn main() -> ! {
    let rcc = device::RCC;
    let flash = device::FLASH;

    // 48 MHz would be more fun.
    flash.acr().modify(|v| {
        v.set_latency(Latency::WS1);
    });
    while flash.acr().read().latency() != Latency::WS1 {
        // spin - should only take a few cycles
    }

    // rcc.cr().modify(|v| {
    //     v.set_hsidiv(Hsidiv::DIV1);
    // });

    rcc.ahbenr().modify(|v| {
        v.set_gpioaen(true);
    });
    cortex_m::asm::dsb(); // likely not necessary

    let gpioa = device::GPIOA;
    gpioa.moder().modify(|v| {
        v.set_moder(5, Moder::OUTPUT);
    });

    loop {
        gpioa.bsrr().write(|w| {
            w.set_bs(5, true);
        });
        cortex_m::asm::delay(12_000_000);
        gpioa.bsrr().write(|w| {
            w.set_br(5, true);
        });
        cortex_m::asm::delay(12_000_000);
    }
}