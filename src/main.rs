use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;
use std::time::Instant;

use rodio::OutputStreamBuilder;
use rodio::{Sink, Source};
use rodio::decoder::DecoderBuilder;

use rppal::gpio::{Gpio, Level, Mode, Bias};
use rppal::gpio::{OutputPin, InputPin, IoPin};

#[allow(unused_imports)]
use rppal::system::DeviceInfo;

#[allow(dead_code)]
fn bit(value: &usize, idx: u32) -> u8 {
    ((value >> idx) & 1) as u8
}

#[allow(dead_code)]
fn get_digital_out(pin: u8) -> OutputPin {
    let io =  Gpio::new().expect("GPIOs not accessible. Input/Output is required.");
    io.get(pin).unwrap().into_output()
}

#[allow(dead_code)]
fn get_digital_in(pin: u8) -> InputPin {
    let io =  Gpio::new().expect("GPIOs not accessible. Input/Output is required.");
    io.get(pin).unwrap().into_input()
}

fn get_digital_generic(pin: u8, mode: Mode) -> IoPin {  
    let io =  Gpio::new().expect("GPIOs not accessible. Input/Output is required.");
    io.get(pin).unwrap().into_io(mode)
}

fn get_bitidx_at_maxdelta(i : &u32, w : &usize, m : u32) -> Option<u64> {
    // for some binary uint w, return the index of the
    // high bit furthest from an arbitrary point i
    let mut d : u32 = 0;
    let mut idx_maxdelta : Option<u64> = None;
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

fn get_decoded_mp3(mf : &str) -> DecoderBuilder<File> {
    let file = File::open(mf).unwrap();
    let len = file.metadata().unwrap().len();
    DecoderBuilder::new()
        .with_data(file)
        .with_hint("mp3")
        .with_byte_len(len)
        .with_seekable(true)
}

fn get_mp3_duration(dat : DecoderBuilder<File>) -> u64 {
    let source = dat.build().unwrap();
    let buffer_duration : Duration = source.total_duration().unwrap();
    let buffer_ms : u64 = buffer_duration.as_millis() as u64;
    return buffer_ms;
}

struct Counter<const BITS: usize> {
    idx : u32,
    pins : [OutputPin; BITS],
}

impl<const BITS: usize>  Counter<BITS> {
    
    fn new(gpio_nums : [u8; BITS]) -> Self {
        let mut outs : Vec<OutputPin> = Vec::new();
        for i in gpio_nums {
            outs.push(get_digital_out(i));
        }
        let pin_outs : [OutputPin; BITS] = outs.try_into().unwrap();
        Self {idx: (1 << BITS) - 1, pins: pin_outs}    
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
      
}

type Counter8 = Counter<3>;

struct Mux8 {
    s : Counter8,
    z : Option<IoPin>,
    e : Option<OutputPin>,
}

fn main() -> Result<(), Box<dyn Error>> {

    // Init the multiplexors to read user's input path for our tape.
    let mut mux_out = Mux8 {
        s : Counter8::new([17, 27, 22]),
        z : None,
        e : Some(get_digital_out(6)),
    };

    mux_out.e.unwrap().set_low();

    let mut mux_in = Mux8 {
        s : Counter8::new([16, 20, 21]),
        z : Some(get_digital_generic(23, Mode::Input)),
        e : None,
    };
    
    mux_in.z.as_mut().unwrap().set_bias(Bias::PullDown);

    let mut mux_in_data : [Level ; 8] = [Level::Low; 8];
        
    // Make a sink containing a loopable, seekable, and measured tape
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream_handle.mixer());

    let mf : String = "assets/music.mp3".to_string();
    let tape = get_decoded_mp3(&mf).build_looped()?;    
    let buffer_ms : u64 = get_mp3_duration(get_decoded_mp3(&mf));

    sink.append(tape);

    // We need to calculate how we want to divi up our tape into seekable chuncks
    let num_chunks : u32  = 8;
    let chunk_len : u64 = buffer_ms / num_chunks as u64;
    let mut jump_to_ms : Option<u64> = None;
   
    loop {
        // Jump if required
        if let Some(j) = jump_to_ms {
            sink.try_seek(Duration::from_millis(j))?;
        }

        // We need to compensate for calculation time so let's take a Instant
        let epoch : Instant = Instant::now();

        // ROR output index and pull all connections to mux_in for that index.
        mux_out.s.up();
        for _ in 0..8 {
            mux_in.s.up();
            sleep(Duration::from_micros(1));
            let reading : Level = mux_in.z.as_mut().unwrap().read();
            mux_in_data[mux_in.s.idx as usize] = reading;
        }

        // Flatten the multiplexed input into a unsigned int.
        let mux_in_byte : usize = mux_in_data.iter().enumerate().fold(
            0, |e, (i, &b)| {                   // let mut e = 0
                e | ((b as u8) as usize) << i   // e |= &b << i;
            }
        );

        // Calculate the longest path we can take between mux_out and mux_in
        let jump_to : Option<u64> = get_bitidx_at_maxdelta(&mux_out.s.idx, &mux_in_byte, 8);

        // Convert that to a jump position on our tape
        jump_to_ms = match jump_to {
            Some(k) => Some(k * chunk_len),
            None => None,
        };

        // Bit of feedback on the console.
        let tape_loc : u64 = sink.get_pos().as_millis() as u64 % buffer_ms;
        println!("{:08}ms -- @{} x{:08b}", tape_loc, mux_out.s.idx, mux_in_byte);

        // Sleep until we are ready to jump again.
        sleep(Duration::from_millis(chunk_len - epoch.elapsed().as_millis() as u64));
    }  
}
