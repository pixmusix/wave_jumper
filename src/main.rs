mod oled;
mod mux;
mod pinio;
mod prelude;

use mux::*;
use oled::*;
use pinio::*;
use prelude::*;


fn bit_at(value: &usize, idx: u32) -> u8 {
    ((value >> idx) & 1) as u8
}

// for some binary uint w, return the index of the
// high bit furthest from an arbitrary point i
fn get_bitidx_at_maxdelta(mark: &u32, value: &usize, modulo: u32) -> Option<u64> {
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
    let mut mux_in_data: [Level; 16] = [Level::Low; 16];

    // A nice oled display for some user feedback
    let mut ssd1306 = Display::new(get_ssd1306(I2c::new()?));

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
    let num_chunks: u32 = 16;
    let mut chunk_len: u64 = buffer_ms / num_chunks as u64;
    let mut jump_to_ms: Option<u64> = None;
    let mut jump_from: u32 = 0;

    // feedback for selected song
    ssd1306.text(5, 5, mf);

    // lets go chaps
    sink.play();

    loop {
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
            ssd1306.rect(0, 0, 128, 20, Some(Brush::Eraser));
            ssd1306.text(5, 5, mf);
        } else if latch == Latch::Set && button.is_high() {
            // User has released the button
            latch = Latch::Reset;
        }
        
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
        let mux_in_byte: usize = mux_in_data.iter().enumerate()
            .fold(0, |e, (i, &b)| {       // let mut e = 0
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

        // Paint any changes to the display
        ssd1306.paint();

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
