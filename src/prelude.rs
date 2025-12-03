// Constructing the oled display
pub use ssd1306::{Ssd1306, I2CDisplayInterface};
pub use ssd1306::size::DisplaySize128x64;
pub use ssd1306::rotation::DisplayRotation;

// Interacting with gpio.
#[allow(unused_imports)]
pub use rpi_pal::gpio::{InputPin, IoPin, OutputPin};
pub use rpi_pal::gpio::{Bias, Level, Mode};

// Constructing an i2c bus
#[allow(unused_imports)]
use rpi_pal::system::DeviceInfo;
pub use rpi_pal::i2c::I2c;

// Can't live with it can't live without it.
pub use std::error::Error;

// Reading files out of ./assets/
pub use std::fs::{File, ReadDir};
pub use std::fs::read_dir;
pub use std::ffi::OsStr;

// Thread sleeping
pub use std::thread::sleep;
pub use std::time::{Duration, Instant};

// Smart pointers
pub use std::rc::Rc;
pub use std::cell::RefCell;

// Music playback
pub use rodio::decoder::DecoderBuilder;
pub use rodio::OutputStreamBuilder;
pub use rodio::{Sink, Source};
pub use cpal::traits::{DeviceTrait, HostTrait};
