use std::ops::Not;
use rppal::gpio::Level;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DotLevel {
     High,
    #[default] Low,
}

impl Not for DotLevel {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            DotLevel::High => DotLevel::Low,
            DotLevel::Low  => DotLevel::High,
        }
    }
}

impl DotLevel {

    #[allow(dead_code)]
    pub fn to_u8(self) -> u8 {
        match self {
            DotLevel::Low => 0,
            DotLevel::High => 1,
        }
    }
    
    #[allow(dead_code)]
    pub fn to_bool(self) -> bool {
        match self {
            DotLevel::Low => false,
            DotLevel::High => true,
        }
    }
    
    #[allow(dead_code)]
    pub fn from_gpio_level(lv: &Level) -> DotLevel {
        match lv {
            Level::High => DotLevel::High,
            Level::Low => DotLevel::Low,
        }
    }
}

// UI for a gpio/mux pin
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Dot {
    pub x: i32,
    pub y: i32,
    pub sz: u32,
    pub lv: DotLevel,
}

impl Dot {
    
    #[allow(dead_code)]
    pub fn same_tile(self, other: &Dot) -> bool {
        self.x == other.x && self.y == other.y
    }

    #[allow(dead_code)]
    pub fn is_low(self) -> bool {
        self.lv == DotLevel::Low
    }

    #[allow(dead_code)]
    pub fn is_high(self) -> bool {
        self.lv == DotLevel::High
    }
}

// UI for connections between pins
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Link {
    pub a: Dot,
    pub b: Dot,
}
