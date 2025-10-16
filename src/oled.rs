use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::Ssd1306;

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, Circle, Line};
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::Text;

use rppal::i2c::I2c;

// 128×64 I²C OLED in buffered-graphics mode
pub type Oled = Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

// Selection of binary PrimitiveStyles that can draw to the oled
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Brush {
    Marker,
    Pen,
    Pencil,
    Eraser,
}

impl Brush {
    pub fn style(self) -> PrimitiveStyle<BinaryColor> {
        use BinaryColor::{Off, On};
        match self {
            Brush::Marker => PrimitiveStyle::with_fill(On),
            Brush::Pen    => PrimitiveStyle::with_stroke(On, 2),
            Brush::Pencil => PrimitiveStyle::with_stroke(On, 1),
            Brush::Eraser => PrimitiveStyle::with_fill(Off),
        }
    }
}

// Models the SSD1306 oled display
pub struct Display {
    oled: Oled,
    default_brush : Brush,
}

impl Display {    
    
    pub fn new(mut disp: Oled) -> Self {
        disp.init().unwrap();
        disp.clear(BinaryColor::Off).unwrap();
        disp.flush().unwrap();

        Display {
            oled: disp,
            default_brush: Brush::Pencil,
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, a: u32, b: u32, brush: Option<Brush>) {
        let draw_point = Point::new(x, y);
        assert!(self.point_in_range(draw_point));
        let size: Size = Size::new(a, b);

        let style: PrimitiveStyle<BinaryColor> = brush.unwrap_or(self.default_brush).style();

        let rect = Rectangle::new(draw_point, size);
        rect.into_styled(style).draw(&mut self.oled).unwrap();
    }

    #[allow(dead_code)]
    pub fn circle(&mut self, x: i32, y: i32, sz: u32, brush: Option<Brush>) {
        let draw_point = Point::new(x, y);
        assert!(self.point_in_range(draw_point));

        let style: PrimitiveStyle<BinaryColor> = brush.unwrap_or(self.default_brush).style();

        let circle = Circle::new(draw_point, sz);
        circle.into_styled(style).draw(&mut self.oled).unwrap();
    }

    #[allow(dead_code)]
    pub fn line(&mut self, x: i32, y: i32, v: i32, w: i32, brush: Option<Brush>) {
        let start = Point::new(x, y);
        let end = Point::new(v, w);
        assert!(self.point_in_range(start) && self.point_in_range(end));
        
        let style: PrimitiveStyle<BinaryColor> = brush.unwrap_or(self.default_brush).style();

        let line = Line::new(start, end);
        line.into_styled(style).draw(&mut self.oled).unwrap();
    }

    #[allow(dead_code)]
    pub fn text(&mut self, x: i32, y: i32, txt: &str) {
        let draw_point = Point::new(x, y);
        assert!(self.point_in_range(draw_point));
        
        let font = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let text = Text::new(&txt, draw_point, font);
        text.draw(&mut self.oled).unwrap();
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.oled.clear(BinaryColor::Off).unwrap();
    }

    #[allow(dead_code)]
    pub fn paint(&mut self) {
        self.oled.flush().unwrap();
    }

    fn point_in_range(&self, pnt: Point) -> bool {
        let oled_size: embedded_graphics::geometry::Size = self.oled.size();
        let x_in_range: bool = (0..oled_size.width).contains(&(pnt.x as u32));
        let y_in_range: bool = (0..oled_size.height).contains(&(pnt.y as u32));
        return x_in_range && y_in_range;
    }
    
}


