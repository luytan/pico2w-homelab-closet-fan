#![no_std]
#![no_main]

use core::future::pending;

use embassy_bmp280::{Bmp280, Bmp280Config};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::block::ImageDef;
use embassy_rp::i2c::Config;
use embassy_rp::peripherals::I2C0;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{
    self as hal,
    i2c::{self},
    peripherals::USB,
    pwm, usb,
};
use embassy_time::Timer;
use log::{info, warn};
//Panic Handler
use panic_probe as _;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

// Interrupts
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Setup usb
    spawner.must_spawn(logger_task(p.USB));
    let setup_fan_pwn_top = || {
        let gpio_pin_fan_top = p.PIN_6;
        let desired_freq_hz = 25_000;
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 16u8;
        let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

        let mut config_pwn_pin_fan_top: pwm::Config = pwm::Config::default();
        config_pwn_pin_fan_top.top = period;
        config_pwn_pin_fan_top.divider = divider.into();

        let pwn_pin_fan_top: pwm::Pwm<'_> = pwm::Pwm::new_output_a(
            p.PWM_SLICE3,
            gpio_pin_fan_top,
            config_pwn_pin_fan_top.clone(),
        );
        pwn_pin_fan_top
    };
    //let pwn_pin_fan_top = setup_fan_pwn_top();
    //let _ = spawner.spawn(top_fan_task(pwn_pin_fan_top));
    Timer::after_secs(3).await;
    // Now setup the temp sensor
    let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, Config::default());
    //
    info!("Attempting BMP280 init");
    let bmp: Bmp280<i2c::I2c<'_, I2C0, i2c::Async>> = match Bmp280::new(
        i2c,
        embassy_bmp280::Bmp280Address::Default,
        Bmp280Config::default(),
    )
    .await
    {
        Ok(b) => {
            info!("BMP280 init OK!");
            b
        }
        Err(err) => loop {
            warn!("bmp init error: {:?}", err);
            Timer::after_secs(5).await;
        },
    };
    let _ = spawner.spawn(monitor_temperature(bmp));
    loop {
        pending::<()>().await;
    }
}

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
#[embassy_executor::task]
async fn top_fan_task(mut pwm: pwm::Pwm<'static>) {
    loop {
        info!("setting to 100%");
        pwm.set_duty_cycle_fully_on().unwrap();
        Timer::after_secs(5).await;
        info!("setting to 80%");
        pwm.set_duty_cycle_percent(80).unwrap();
        Timer::after_secs(5).await;
        info!("setting to 60%");
        pwm.set_duty_cycle_percent(60).unwrap();
        Timer::after_secs(5).await;
        info!("setting to 40%");
        pwm.set_duty_cycle_percent(40).unwrap();
        Timer::after_secs(5).await;
        info!("setting to 20%");
        pwm.set_duty_cycle_percent(20).unwrap();
        Timer::after_secs(5).await;
        info!("setting to 0%");
        pwm.set_duty_cycle_fully_off().unwrap();
        Timer::after_secs(10).await;
    }
}

#[embassy_executor::task]
async fn monitor_temperature(mut bmp: Bmp280<i2c::I2c<'static, I2C0, i2c::Async>>) {
    loop {
        if let Ok(data) = bmp.read().await {
            let temp: f32 = data.temperature_cdeg as f32 / 100 as f32;
            info!("temp = {}c", temp);
        }
        Timer::after_millis(500).await;
    }
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"pico-fan"),
    embassy_rp::binary_info::rp_program_description!(c"your program description"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];
