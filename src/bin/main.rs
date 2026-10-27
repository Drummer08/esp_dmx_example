#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcCalCurve, AdcConfig, Attenuation};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_println as _;

use esp_dmx::{Channel, Dmx};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const ADC_MIN: u16 = 10; // mV, small margin above 0
const ADC_MAX: u16 = 3250; // mV, small margin below your measured ~3292

fn adc_to_dmx(mv: u16) -> u8 {
    let r = mv.clamp(ADC_MIN, ADC_MAX) - ADC_MIN;
    ((r as u32 * 255) / (ADC_MAX - ADC_MIN) as u32) as u8
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32c6 -o esp32c6-wroom-1 -o unstable-hal -o defmt -o esp-backtrace -o zed

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO4
    // - GPIO5
    // - GPIO8
    // - GPIO9
    // - GPIO15
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _gpio24 = peripherals.GPIO24;
    let _gpio25 = peripherals.GPIO25;
    let _gpio26 = peripherals.GPIO26;
    let _gpio27 = peripherals.GPIO27;
    let _gpio28 = peripherals.GPIO28;
    let _gpio29 = peripherals.GPIO29;
    let _gpio30 = peripherals.GPIO30;

    let mut adc1_config = AdcConfig::new();
    let mut pin =
        adc1_config.enable_pin_with_cal::<_, AdcCalCurve<_>>(peripherals.GPIO0, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let delay = Delay::new();

    let mut dmx = Dmx::with_defaults(peripherals.UART1, peripherals.GPIO4).unwrap();

    // set dimmer and rgb channel
    dmx.set_channels(Channel::new(1).unwrap(), &[255, 255, 0, 150])
        .unwrap();
    info!("DMX initialised");

    loop {
        dmx.send_frame(&delay).unwrap();
        let mv = nb::block!(adc1.read_oneshot(&mut pin)).unwrap();
        // info!("{}", adc_to_dmx(mv));
        dmx.set_channel(Channel::new(1).unwrap(), adc_to_dmx(mv));
    }
}
