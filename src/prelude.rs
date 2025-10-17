// Constructing the oled display
pub use ssd1306::{Ssd1306, I2CDisplayInterface};
pub use ssd1306::size::DisplaySize128x64;
pub use ssd1306::rotation::DisplayRotation;

// Interacting with gpio.
#[allow(unused_imports)]
pub use rppal::gpio::{InputPin, IoPin, OutputPin};
pub use rppal::gpio::{Bias, Level, Mode};

// Constructing an i2c bus
#[allow(unused_imports)]
use rppal::system::DeviceInfo;
pub use rppal::i2c::I2c;

// Can't live with 'em; can't live without 'em.
pub use std::error::Error;
pub use std::ops::Not;

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
