#[cfg(all(not(esp_idf_version_major = "4"), not(esp_idf_version = "5.1")))]
pub mod modern;

use core::time::Duration;

use crate::gpio::*;
use crate::peripheral::Peripheral;
use crate::units::*;

pub use embedded_hal::i2c::Operation;
use esp_idf_sys::i2c_port_t;

#[cfg(all(not(esp_idf_version_major = "4"), not(esp_idf_version = "5.1")))]
#[repr(u8)]
enum UsedDriver {
    None = 0,
    Legacy = 1,
    Modern = 2,
}

#[cfg(all(not(esp_idf_version_major = "4"), not(esp_idf_version = "5.1")))]
// 0 -> no driver, 1 -> legacy driver, 2 -> modern driver
static DRIVER_IN_USE: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(UsedDriver::None as u8);

#[cfg(all(not(esp_idf_version_major = "4"), not(esp_idf_version = "5.1")))]
fn check_and_set_modern_driver() {
    match DRIVER_IN_USE.compare_exchange(
        UsedDriver::None as u8,
        UsedDriver::Modern as u8,
        core::sync::atomic::Ordering::Relaxed,
        core::sync::atomic::Ordering::Relaxed,
    ) {
        Err(e) if e == UsedDriver::Legacy as u8 => panic!("Legacy I2C driver is already in use. Either legacy driver or modern driver can be used at a time."),
        _ => ()
    }
}

fn check_and_set_legacy_driver() {
    match DRIVER_IN_USE.compare_exchange(
            UsedDriver::None as u8,
            UsedDriver::Legacy as u8,
            core::sync::atomic::Ordering::Relaxed,
            core::sync::atomic::Ordering::Relaxed,
    ) {
            Err(e) if e == UsedDriver::Modern as u8 => panic!("Modern I2C driver is already in use. Either legacy driver or modern driver can be used at a time."),
            _ => ()
    }
}

crate::embedded_hal_error!(
    I2cError,
    embedded_hal::i2c::Error,
    embedded_hal::i2c::ErrorKind
);

const APB_TICK_PERIOD_NS: u32 = 1_000_000_000 / 80_000_000;
#[derive(Copy, Clone, Debug)]
pub struct APBTickType(::core::ffi::c_int);
impl From<Duration> for APBTickType {
    fn from(duration: Duration) -> Self {
        APBTickType(
            ((duration.as_nanos() + APB_TICK_PERIOD_NS as u128 - 1) / APB_TICK_PERIOD_NS as u128)
                as ::core::ffi::c_int,
        )
    }
}

pub trait I2c: Send {
    fn port() -> i2c_port_t;
}

macro_rules! impl_i2c {
    ($i2c:ident: $port:expr) => {
        crate::impl_peripheral!($i2c);

        impl I2c for $i2c {
            #[inline(always)]
            fn port() -> i2c_port_t {
                $port
            }
        }
    };
}

impl_i2c!(I2C0: 0);
#[cfg(not(any(esp32c3, esp32c2, esp32c6)))]
impl_i2c!(I2C1: 1);
