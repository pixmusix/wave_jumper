#[allow(unused_imports)]
use rpi_pal::gpio::{InputPin, IoPin, OutputPin};
use rpi_pal::gpio::{Gpio, Mode};

#[allow(dead_code)]
pub fn get_digital_out(pin: u8) -> OutputPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_output()
}

#[allow(dead_code)]
pub fn get_digital_in(pin: u8) -> InputPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_input()
}

#[allow(dead_code)]
pub fn get_digital_generic(pin: u8, mode: Mode) -> IoPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_io(mode)
}
