#![no_std]
#![no_main]

// Halt when the program panics.
extern crate panic_halt;

// Includes.
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use stm32f3::stm32f303;

#[entry]
fn main() -> ! {
// Set up SysTick peripheral.
let cm_p = cortex_m::Peripherals::take().unwrap();
let mut syst = cm_p.SYST;
syst.set_clock_source( SystClkSource::Core );
// ~1s period; STM32L0 boots to a ~2.1MHz internal oscillator.
syst.set_reload( 2_100_000 );
syst.enable_counter();

// Set up GPIO pin PA5 as push-pull output.
let p = stm32f303::Peripherals::take().unwrap();
let rcc = p.RCC;
rcc.ahbenr.write(|w| w.iopaen().set_bit());
rcc.ahbenr.write(|w| w.iopcen().set_bit());
let gpioa = p.GPIOA;
let gpioc = p.GPIOC;
unsafe { gpioa.moder.write( |w| w.moder5().bits( 0b01 ) ); }
unsafe { gpioc.moder.write( |w| w.moder13().bits( 0b00 ) ); }
gpioa.otyper.write( |w| w.ot5().clear_bit() );

// Restart the SysTick counter.
syst.clear_current();

// Main loop.
loop {
  if(gpioc.idr.read().idr13().bit_is_set() == true ) {
    gpioa.odr.write( |w| w.odr5().set_bit() );
  } else {
    gpioa.odr.write( |w| w.odr5().clear_bit() );
  }
}
}