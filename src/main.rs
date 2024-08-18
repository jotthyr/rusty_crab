#![no_std]
#![no_main]

mod fmt;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use fmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embassy_stm32::{exti::ExtiInput, gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed}};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;

pub struct Unit {
    pub millimeters: f64,
    pub centimeters: f64,
    pub decimeters: f64,
    pub meters: f64,
}

// Declare async tasks
#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, Speed::Low);

    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.set_high();
        Timer::after_millis(1500).await;
        led.set_low();
        Timer::after_millis(1500).await;
    }
}

// Main is itself an async task as well.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let ch1 = PwmPin::new_ch1(p.PC0, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM1, Some(ch1), None, None, None, hz(50), Default::default());
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);

    // Spawned tasks run in the background, concurrently.
    spawner.spawn(blink(p.PA5.degrade())).unwrap();

    let mut button = ExtiInput::new(Input::new(p.PC13, Pull::Up), p.EXTI13);

    let mut trigger = Output::new(p.PB8, Level::Low, Speed::Low);
    let mut echo = ExtiInput::new(Input::new(p.PB9, Pull::Up), p.EXTI9);

    let mut inc = 0;
    const SPEED_OF_SOUND: f64 = 0.0343;

    pwm.set_duty(Channel::Ch1, max / 40);

    fn calculate_speed(duration: Duration) -> Unit {
        // cannot calculate distance if no object is
        // detected between 100uS - 18mS
        if duration.as_micros() < 100 || duration.as_millis() > 18 {
            return Unit {
                millimeters: 4000.0,
                centimeters: 400.0,
                decimeters: 40.0,
                meters: 4.0,
            };
        }

        // divide by 2 since the signal travels
        // to the object and back

        let distance = (SPEED_OF_SOUND * (duration.as_micros() as f64)) / 2f64;

        // cannot be lower than 2cm
        if distance < 2.0 {
            return Unit {
                millimeters: 0.0,
                centimeters: 0.0,
                decimeters: 0.0,
                meters: 0.0,
            };
        }

        // sensor has a maximum range of 400cm / 4m
        if distance > 400.0 {
            return Unit {
                millimeters: 4000.0,
                centimeters: 400.0,
                decimeters: 40.0,
                meters: 4.0,
            };
        }

        return Unit {
            millimeters: distance * 10.0,
            centimeters: distance,
            decimeters: distance / 10.0,
            meters: distance / 100.0,
        };
    }

    loop {

        info!("{}",pwm.get_max_duty());
        // Asynchronously wait for GPIO events, allowing other tasks
        // to run, or the core to sleep.
        trigger.set_low();
        Timer::after(Duration::from_millis(10)).await;
        trigger.set_high();
        Timer::after(Duration::from_micros(10)).await;
        trigger.set_low();

        info!("debug 1");

        echo.wait_for_high().await;
        let instant = Instant::now();
        echo.wait_for_low().await;

        info!("debug 2");

        let duration = Instant::checked_duration_since(&Instant::now(), instant).unwrap();

        Timer::after_millis(100).await;
        pwm.set_duty(Channel::Ch1, calculate_speed(duration).centimeters as u16 * 100);

        info!("debug {}", calculate_speed(duration).centimeters);
        info!("debug {}", duration);

    }
}
