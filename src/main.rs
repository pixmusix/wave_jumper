mod oled;
mod mux;
mod pinio;
mod prelude;

use mux::*;
use oled::*;
use pinio::*;
use prelude::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum DotLevel {
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
    fn to_u8(self) -> u8 {
        match self {
            DotLevel::Low => 0,
            DotLevel::High => 1,
        }
    }
    
    fn to_bool(self) -> bool {
        match self {
            DotLevel::Low => false,
            DotLevel::High => true,
        }
    }
    
    fn from_gpio_level(lv: &Level) -> DotLevel {
        match lv {
            Level::High => DotLevel::High,
            Level::Low => DotLevel::Low,
        }
    }
}

// UI for a gpio/mux pin
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Dot {
    x: i32,
    y: i32,
    sz: u32,
    lv: DotLevel,
}

impl Dot {
    fn same_tile(self, other: &Dot) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn is_low(self) -> bool {
        self.lv == DotLevel::Low
    }

    fn is_high(self) -> bool {
        self.lv == DotLevel::High
    }
}

// UI for connections between pins
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
struct Link {
    a: Dot,
    b: Dot,
}

fn bit_at(value: &u16, idx: u32) -> u8 {
    ((value >> idx) & 1) as u8
}

// for some binary number (value), return the index of the
// high bit furthest from an arbitrary point in this bitstring (mark)
fn get_bitidx_at_maxdelta(mark: &u32, value: &u16, modulo: u32) -> Option<u64> {
    let mut delta: u32 = 0;
    let mut idx_maxdelta: Option<u64> = None;
    while delta < (modulo / 2) {
        delta += 1;
        let lft: u32 = mark.wrapping_sub(delta) % modulo;
        let rht: u32 = mark.wrapping_add(delta) % modulo;
        if bit_at(value, lft) != 0 {
            idx_maxdelta = Some(lft as u64);
        }
        if bit_at(value, rht) != 0 {
            idx_maxdelta = Some(rht as u64);
        }
    }
    return idx_maxdelta;
}

fn get_mp3_from_local_assets() -> Result<Vec<String>, std::io::Error> {
    let entries: ReadDir = read_dir("./assets/")?;
    let ok_entries = entries.filter_map(|res| res.ok());
    let paths = ok_entries.map(|e| e.path());
    
    let mp3s = paths.filter(|p| {
        let ext: Option<&OsStr> = p.extension();
        let ext_str: Option<&str> = ext.and_then(|e| e.to_str());
        ext_str.map_or(false, |e| e.eq_ignore_ascii_case("mp3"))
    });

    let mp3_string = mp3s.map(|p| p.to_string_lossy().into_owned());
    Ok(mp3_string.collect())
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

fn punch_file_into_sink(mf: &str, sink: &Sink) -> Result<(), Box<dyn Error>>{
    let tape = get_decoded_mp3(&mf).build_looped()?;
    sink.append(tape);
    Ok(())
}

fn get_ssd1306(i2c: I2c) -> Oled {
    let interface = I2CDisplayInterface::new(i2c);
    let oled = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);
    oled.into_buffered_graphics_mode()
}

fn get_dot_row(doty: i32, size: u32, pad: u32, num: usize) -> Vec<Dot> {
    let mut dots: Vec<Dot> = Vec::new();
    let mut dotx: i32 = 0; 
    for _ in 0..num {
        dotx = (dotx as u32 + pad) as i32;
        dots.push(Dot{x: dotx, y: doty, sz: size, lv: DotLevel::Low});
    }
    return dots;
}

fn clear_dot(oled: &mut Display, dot: &mut Dot) {
    dot.lv = DotLevel::Low;
    oled.circle(dot.x, dot.y, dot.sz, Some(Brush::Eraser));
    oled.circle(dot.x, dot.y, dot.sz, Some(Brush::Pen));
}

fn fill_dot(oled: &mut Display, dot: &mut Dot) {
    dot.lv = DotLevel::High;
    oled.circle(dot.x, dot.y, dot.sz, Some(Brush::Marker));
}
    
fn main() -> Result<(), Box<dyn Error>> {
    // Let's ensure we have a sink before we proceed
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no audio devices.. gross!");
    let dev_conf = device.default_output_config()?;
    let dev_name = device.name().unwrap_or_else(|_| "dead device _ do not use".into());
    println!("@Device Name = {}", dev_name);
    println!("@Device Config = {:?}", dev_conf);

    // Number of chunks to split our song into.
    const num_steps: usize = 16;

    /* Init the multiplexors to read user's input path for our tape */
    // Construct output mux
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
    // An array to store our output mux
    let mut mux_out: [Mux8; 2] = [mux_out_lsb, mux_out_msb];
    for mx in mux_out.iter_mut() {
        mx.e.as_mut().unwrap().set_high();
    }
    // Construct Input Mux
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
    // Array to store out input mux
    let mut mux_in: [Mux8; 2] = [mux_in_msb, mux_in_lsb];
    for mx in mux_in.iter_mut() {
        mx.z.as_mut().unwrap().set_bias(Bias::PullDown);
    }

    // This array can collect and store the data received from every input mux
    let mut mux_in_data: [Level; num_steps] = [Level::Low; num_steps];

    // A nice oled display for some user feedback
    let mut ssd1306 = Display::new(get_ssd1306(I2c::new()?));

    // Some UI decisions.
    const title_ui_ycoord: i32 = 5;
    const title_ui_ysize: u32 = 8;
    const muxout_ui_ycoord: i32 = (title_ui_ycoord as u32 + title_ui_ysize + 2) as i32;
    const muxin_ui_ycoord: i32 = 55;
    const line_ui_ystart: i32 = muxout_ui_ycoord + 10;
    const line_ui_yend: i32 = muxin_ui_ycoord - 5;
    const dot_ui_size: u32 = 7;
    const dot_ui_xpad: u32 = dot_ui_size;

    // we need a to keep track of the connections.
    let mut links: Vec<Link> = Vec::new();

    // Some user input to skip to next song
    let button : InputPin = get_digital_in(26);
    let mut latch = Latch::Reset;

    // Make a sink containing a loopable, seekable, and measured tape
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream_handle.mixer());

    // Pull the mp3s from ./assets/
    let mp3s: Vec<String> = get_mp3_from_local_assets()?;
    let mut current_mp3_idx: usize = 0;
    let mut mf: &String = &mp3s[current_mp3_idx];
    let mut buffer_ms: u64 = get_mp3_duration(get_decoded_mp3(mf));

    // Fill the sink with some sound!
    punch_file_into_sink(mf, &sink)?;

    // We need to calculate how we want to divi up our tape into seekable chuncks
    const num_chunks : u32 = num_steps as u32;
    let mut chunk_len: u64 = buffer_ms / num_chunks as u64;
    let mut jump_to: Option<u64> = None;
    let mut jump_to_ms: Option<u64> = None;
    let mut position: u32 = 0;

    // feedback for selected song
    ssd1306.text(5, title_ui_ycoord, mf);
    // feedback for outmux
    let mut muxout_dots: [Dot; num_steps] = get_dot_row(
        muxout_ui_ycoord,
        dot_ui_size,
        dot_ui_xpad,
        num_steps
    ).try_into().unwrap();
    for dot in muxout_dots {
        ssd1306.circle(dot.x, dot.y, dot.sz, Some(Brush::Pen));
    }
    // feedback for inmux
    let mut muxin_dots: [Dot; num_steps] = get_dot_row(
        muxin_ui_ycoord,
        dot_ui_size,
        dot_ui_xpad,
        num_steps
    ).try_into().unwrap();
    for dot in muxin_dots {
        ssd1306.circle(dot.x, dot.y, dot.sz, Some(Brush::Pen));
    }

    // lets go chaps
    sink.play();

    loop {
        // We need to compensate for calculation time so let's take a Instant
        let epoch: Instant = Instant::now();
        
        // Check if we skip to next tape loop
        if button.is_low() && latch == Latch::Reset {
            // Set the latch to avoid double skipping
            latch = Latch::Set;
            // Clear sink of our current loop
            sink.stop();
            // Get the next song
            current_mp3_idx = (current_mp3_idx + 1) % mp3s.len();
            mf = &mp3s[current_mp3_idx];
            // We need to update the length of each sample to new track
            buffer_ms = get_mp3_duration(get_decoded_mp3(mf));
            chunk_len = buffer_ms / num_chunks as u64;
            // Got a new file, let's play it
            punch_file_into_sink(mf, &sink)?;
            sink.play();

            // Draw the music file path to screen
            ssd1306.rect(0, 0, 128, title_ui_ysize, Some(Brush::Eraser));
            ssd1306.text(5, 5, mf);
        } else if latch == Latch::Set && button.is_high() {
            // User has released the button
            latch = Latch::Reset;
        }
        
        // Clear dot from last time
        let mut muxout_dot: &mut Dot = &mut muxout_dots[position as usize];
        clear_dot(&mut ssd1306, muxout_dot);
                
        // Jump if required
        if let Some(j) = jump_to_ms {
            sink.try_seek(Duration::from_millis(j))?;
        }

        // Update our position
        if let Some(j) = jump_to {
            position = j as u32;
        } else {
            position = (position + 1) % num_chunks;
        }
               
        // Throw our position onto the GPIO
        let inv_position: u32 = num_chunks - position - 1;
        let (w, i): (usize, u32) = ((inv_position / 8) as usize, inv_position % 8);
        mux_out[w].s.borrow_mut().set(i)?;
        mux_out[w].e.as_mut().unwrap().set_low();

        // Replace with new dot after ROR
        muxout_dot = &mut muxout_dots[position as usize];
        fill_dot(&mut ssd1306, muxout_dot);
        
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

        // Update the state of our dots.
        for (mut dot, &dat) in muxin_dots.iter_mut().zip(&mux_in_data) {
            let cache: Dot = dot.clone();
            dot.lv = DotLevel::from_gpio_level(&dat);
            if *dot != cache {
                match dot.lv {
                    DotLevel::High => fill_dot(&mut ssd1306, dot),
                    DotLevel::Low => clear_dot(&mut ssd1306, dot),
                }
            }
        }

        println!("dots drawn {:?}", epoch.elapsed());

        // Lets remove links where link.b does not share its state with muxin_dots
        let linkscache = links.clone();
        links.retain(|lk| {
            let valid_lka: bool = lk.a.same_tile(muxout_dot);
            let in_high: bool = muxin_dots.iter().any(|d| d.same_tile(&lk.b) && d.is_high());
            !valid_lka && !in_high
        });

        // We need to rebuild all the connections for this muxout_dot
        links.extend(
            muxin_dots.iter().filter(|d| d.is_high())
                .map(|d| Link { a: *muxout_dot, b: *d }),
        );

        // There with be douplicates, let's drop them
        links.sort_by_key(|lk| (lk.a, lk.b));
        links.dedup_by(|outdot, indot| {
            outdot.a.same_tile(&indot.a) && outdot.b.same_tile(&indot.b)
        });

        // Let's erase all drawn connections and redraw them, there won't be many.
        if linkscache != links {
            println!("Erase");
            ssd1306.rect(
                0, line_ui_ystart,
                128, (line_ui_yend - line_ui_ystart + 2) as u32,
                Some(Brush::Eraser)
            );
            for lk in links.iter() {
                ssd1306.line(lk.a.x, line_ui_ystart, lk.b.x, line_ui_yend, None);
            }
        }
        println!("links drawn {:?}", epoch.elapsed());

        // Flatten the multiplexed input into a unsigned int.
        let mux_in_word: u16 = mux_in_data.iter().enumerate()
            .fold(0, | word, (i, &byte) | {
                // let k : u16 = (u16::BITS - 1 - i as u32) as u16;
                word | (byte as u16) << i
        });
    
        // Calculate the longest path we can take between mux_out and mux_in
        jump_to = get_bitidx_at_maxdelta(&position, &mux_in_word, num_chunks);

        // Convert that to a jump position on our tape
        jump_to_ms = match jump_to {
            Some(k) => Some(k * chunk_len),
            None => None,
        };

        // Paint any changes to the display
        ssd1306.paint();
        
        println!("painted {:?}", epoch.elapsed());

        // Bit of feedback on the console.
        let tape_loc: u64 = sink.get_pos().as_millis() as u64 % buffer_ms;
        println!(
            "{:08}ms -- @{:02} x{:016b}",
            tape_loc, position, mux_in_word
        );

        // Sleep until we are ready to jump again.
        let chunk_duration = Duration::from_millis(chunk_len);
        let looptime_remaining: Duration = chunk_duration.saturating_sub(epoch.elapsed());
        println!("loop time remaining {:?}", looptime_remaining);
        sleep(looptime_remaining);
    }
}
