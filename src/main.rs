#![no_std]
#![no_main]

use cortex_m::prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_WriteRead};
use embassy_bmp280::{Bmp280, Bmp280Address, Bmp280Config};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::I2C0;
use embassy_rp::{
    self as hal,
    gpio::{Input, Pull},
    i2c::{self, Config},
    peripherals::USB,
    usb,
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

    let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, Config::default());

    // Wait for USB serial
    Timer::after_secs(2).await;

    info!("Attempting BMP280 init");
    let mut bmp = match Bmp280::new(i2c, Bmp280Address::Default, Bmp280Config::default()).await {
        Ok(b) => {
            info!("BMP280 init OK!");
            b
        }
        Err(err) => loop {
            warn!("bmp init error: {:?}", err);
            Timer::after_secs(5).await;
        },
    };
    loop {
        Timer::after_secs(1).await;
        if let Ok(data) = bmp.read().await {
            let temp: i32 = data.temperature_cdeg;
            info!("data: {:?}", temp);
        } else {
            warn!("couldn't read data");
        }
    }
}

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
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
