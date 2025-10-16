use crate::pinio::*;

#[allow(unused_imports)]
use rppal::gpio::{InputPin, IoPin, OutputPin};
use rppal::gpio::Level;

use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

// 3 bit binary counter
pub type Counter8 = Counter<3>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Latch {Set, Reset}

// Binary counter that punches onto GPIO
#[derive(Debug)]
pub struct Counter<const BITS: usize> {
    pub idx: u32,
    pins: [OutputPin; BITS],
}

impl<const BITS: usize> Counter<BITS> {
    pub fn new(gpio_nums: [u8; BITS]) -> Self {
        let mut outs: Vec<OutputPin> = Vec::new();
        for i in gpio_nums {
            outs.push(get_digital_out(i));
        }
        let pin_outs: [OutputPin; BITS] = outs.try_into().unwrap();
        Self {
            idx: (1 << BITS) - 1,
            pins: pin_outs,
        }
    }

    fn out(&mut self) {
        let mut b = self.idx;
        for pin in self.pins.iter_mut() {
            pin.write(Level::from(b & 1 != 0));
            b >>= 1;
        }
    }

    pub fn up(&mut self) {
        self.idx = (self.idx + 1) % (1 << BITS);
        self.out();
    }

    pub fn set(&mut self, idx: u32) -> Result<(), Box<dyn Error>> {
        if idx >= (1 << BITS) {
            return Err("The provided index is unexpressible by this counter".into());
        }
        self.idx = idx;
        self.out();
        Ok(())
    }
}

// Models a 74HC4051 multiplexor
#[derive(Debug)]
pub struct Mux8 {
    pub s: Rc<RefCell<Counter8>>,
    pub z: Option<IoPin>,
    pub e: Option<OutputPin>,
}
