use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use rodio::decoder::DecoderBuilder;
use rodio::OutputStreamBuilder;
use rodio::{Sink, Source};

use cpal::traits::{DeviceTrait, HostTrait};

use rppal::gpio::{Bias, Gpio, Level, Mode};
use rppal::gpio::{InputPin, IoPin, OutputPin};
use rppal::i2c::I2c;

use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};
use embedded_graphics::text::Text;

// 128×64 I²C OLED in buffered-graphics mode
type Oled = Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

// 3 bit binary counter
type Counter8 = Counter<3>;

#[allow(unused_imports)]
use rppal::system::DeviceInfo;

#[allow(dead_code)]
fn bit(value: &usize, idx: u32) -> u8 {
    ((value >> idx) & 1) as u8
}

#[allow(dead_code)]
fn get_digital_out(pin: u8) -> OutputPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_output()
}

#[allow(dead_code)]
fn get_digital_in(pin: u8) -> InputPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_input()
}

fn get_digital_generic(pin: u8, mode: Mode) -> IoPin {
    let io = Gpio::new().expect("GPIOs not accessible.");
    io.get(pin).unwrap().into_io(mode)
}

fn get_bitidx_at_maxdelta(i: &u32, w: &usize, m: u32) -> Option<u64> {
    // for some binary uint w, return the index of the
    // high bit furthest from an arbitrary point i
    let mut d: u32 = 0;
    let mut idx_maxdelta: Option<u64> = None;
    while d < (m / 2) {
        d += 1;
        let sin: u32 = i.wrapping_sub(d) % m;
        let dex: u32 = i.wrapping_add(d) % m;
        if bit(w, sin) != 0 {
            idx_maxdelta = Some(sin as u64);
        }
        if bit(w, dex) != 0 {
            idx_maxdelta = Some(dex as u64);
        }
    }
    return idx_maxdelta;
}

fn get_decoded_mp3(mf: &str) -> DecoderBuilder<File> {
    let file = File::open(mf).unwrap();
    let len = file.metadata().unwrap().len();
    DecoderBuilder::new()
        .with_data(file)
        .with_hint("mp3")
        .with_byte_len(len)
        .with_seekable(true)
}

fn get_mp3_duration(dat: DecoderBuilder<File>) -> u64 {
    let source = dat.build().unwrap();
    let buffer_duration: Duration = source.total_duration().unwrap();
    let buffer_ms: u64 = buffer_duration.as_millis() as u64;
    return buffer_ms;
}

fn get_ssd1306(i2c: I2c) -> Oled {
    let interface = I2CDisplayInterface::new(i2c);
    let oled = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);
    oled.into_buffered_graphics_mode()
}

struct Display {
    oled: Oled,
}

impl Display {
    fn new(mut disp: Oled) -> Self {
        disp.init().unwrap();
        disp.clear(BinaryColor::Off).unwrap();
        disp.flush().unwrap();

        Display { oled: disp }
    }

    fn point_in_range(&self, pnt: Point) -> bool {
        let oled_size: embedded_graphics::geometry::Size = self.oled.size();
        let x_in_range: bool = (0..oled_size.width).contains(&(pnt.x as u32));
        let y_in_range: bool = (0..oled_size.height).contains(&(pnt.y as u32));
        return x_in_range && y_in_range;
    }

    fn circle(
        &mut self,
        x: i32,
        y: i32,
        sz: u32,
        solid: bool,
        viz: bool,
    ) -> Result<(), Box<dyn Error>> {
        use BinaryColor::{Off, On};
        let draw_point = Point::new(x, y);
        if !self.point_in_range(draw_point) {
            return Err("Cannot draw circle outside the bounds of a the display<128,64>".into());
        }

        let circle = Circle::new(draw_point, sz);

        let style: PrimitiveStyle<BinaryColor> = if solid {
            PrimitiveStyle::with_fill(if viz { On } else { Off })
        } else {
            PrimitiveStyle::with_stroke(if viz { On } else { Off }, 1)
        };

        circle.into_styled(style).draw(&mut self.oled).unwrap();
        Ok(())
    }

    fn line(&mut self, x: i32, y: i32, v: i32, w: i32) {
        let line = Line::new(Point::new(x, y), Point::new(v, w));
        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        line.into_styled(style).draw(&mut self.oled).unwrap();
    }

    fn text(&mut self, x: i32, y: i32, txt: String) {
        let font = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let text = Text::new(&txt, Point::new(10, 20), font);
        text.draw(&mut self.oled).unwrap();
    }

    fn clear(&mut self) {
        self.oled.clear(BinaryColor::Off).unwrap();
    }

    fn paint(&mut self) {
        self.oled.flush().unwrap();
    }
}

struct Counter<const BITS: usize> {
    idx: u32,
    pins: [OutputPin; BITS],
}

impl<const BITS: usize> Counter<BITS> {
    fn new(gpio_nums: [u8; BITS]) -> Self {
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

    fn up(&mut self) {
        self.idx = (self.idx + 1) % (1 << BITS);
        self.out();
    }

    fn set(&mut self, idx: u32) -> Result<(), Box<dyn Error>> {
        if idx >= (1 << BITS) {
            return Err("The provided index is unexpressible by this counter".into());
        }
        self.idx = idx;
        self.out();
        Ok(())
    }
}

struct Mux8 {
    s: Rc<RefCell<Counter8>>,
    z: Option<IoPin>,
    e: Option<OutputPin>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Let's ensure we have a sink before we proceed
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no audio devices.. gross!");
    let dev_conf = device.default_output_config()?;

    println!(
        "name = {}",
        device
            .name()
            .unwrap_or_else(|_| "dead device _ do not use".into())
    );
    println!("{:?}", dev_conf);

    // Init the multiplexors to read user's input path for our tape.
    let counter_muxout = Counter8::new([17, 27, 22]);
    let mutrc_counter_muxout = Rc::new(RefCell::new(counter_muxout));
    let mux_out_lsb = Mux8 {
        s: Rc::clone(&mutrc_counter_muxout),
        z: None,
        e: Some(get_digital_out(5)),
    };
    let mux_out_msb = Mux8 {
        s: Rc::clone(&mutrc_counter_muxout),
        z: None,
        e: Some(get_digital_out(6)),
    };

    let mut mux_out: [Mux8; 2] = [mux_out_lsb, mux_out_msb];
    for mx in mux_out.iter_mut() {
        mx.e.as_mut().unwrap().set_high();
    }

    let counter_muxin = Counter8::new([21, 20, 16]);
    let mutrc_counter_muxin = Rc::new(RefCell::new(counter_muxin));
    let mux_in_lsb = Mux8 {
        s: Rc::clone(&mutrc_counter_muxin),
        z: Some(get_digital_generic(23, Mode::Input)),
        e: None,
    };
    let mux_in_msb = Mux8 {
        s: Rc::clone(&mutrc_counter_muxin),
        z: Some(get_digital_generic(24, Mode::Input)),
        e: None,
    };

    let mut mux_in: [Mux8; 2] = [mux_in_msb, mux_in_lsb];
    for mx in mux_in.iter_mut() {
        mx.z.as_mut().unwrap().set_bias(Bias::PullDown);
    }

    let mut mux_in_data: [Level; 16] = [Level::Low; 16];

    // A nice oled display for some user feedback
    let mut ssd1306 = Display::new(get_ssd1306(I2c::new()?));
    ssd1306.circle(64, 32, 5, true, true);
    ssd1306.paint();

    // Make a sink containing a loopable, seekable, and measured tape
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream_handle.mixer());

    let mf: String = "assets/arp.mp3".to_string();
    let tape = get_decoded_mp3(&mf).build_looped()?;
    let buffer_ms: u64 = get_mp3_duration(get_decoded_mp3(&mf));

    sink.append(tape);

    // We need to calculate how we want to divi up our tape into seekable chuncks
    let num_chunks: u32 = 16;
    let chunk_len: u64 = buffer_ms / num_chunks as u64;
    let mut jump_to_ms: Option<u64> = None;
    let mut jump_from: u32 = 0;

    loop {
        // Jump if required
        if let Some(j) = jump_to_ms {
            sink.try_seek(Duration::from_millis(j))?;
        }

        // We need to compensate for calculation time so let's take a Instant
        let epoch: Instant = Instant::now();

        // ROR output index
        jump_from = (jump_from + 1) % num_chunks;
        let (w, i): (usize, u32) = ((jump_from / 8) as usize, jump_from % 8);
        mux_out[w].s.borrow_mut().set(i)?;
        mux_out[w].e.as_mut().unwrap().set_low();

        // Scan all multiplexed inputs
        for (k, mx) in mux_in.iter_mut().enumerate() {
            for _ in 0..8 {
                mx.s.borrow_mut().up();
                sleep(Duration::from_micros(1000));
                let reading: Level = mx.z.as_mut().unwrap().read();
                let i: usize = (k * 8) + mx.s.borrow().idx as usize;
                mux_in_data[i] = reading;
            }
        }

        // We're done with our IO so we can diable the mux again.
        mux_out[w].e.as_mut().unwrap().set_high();

        // Flatten the multiplexed input into a unsigned int.
        let mux_in_byte: usize = mux_in_data.iter().enumerate().fold(0, |e, (i, &b)| {
            // let mut e = 0
            e | ((b as u8) as usize) << i // e |= &b << i;
        });

        // Calculate the longest path we can take between mux_out and mux_in
        let jump_to: Option<u64> = get_bitidx_at_maxdelta(&jump_from, &mux_in_byte, num_chunks);

        if let Some(j) = jump_to {
            jump_from = j as u32;
        }

        // Convert that to a jump position on our tape
        jump_to_ms = match jump_to {
            Some(k) => Some(k * chunk_len),
            None => None,
        };

        // Bit of feedback on the console.
        let tape_loc: u64 = sink.get_pos().as_millis() as u64 % buffer_ms;
        println!(
            "{:08}ms -- @{:02} x{:016b}",
            tape_loc, jump_from, mux_in_byte
        );

        // Sleep until we are ready to jump again.
        sleep(Duration::from_millis(chunk_len) - epoch.elapsed());
    }
}
