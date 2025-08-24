use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;

use rand::Rng;

use rodio::OutputStreamBuilder;
use rodio::{Sink, Source};
use rodio::decoder::DecoderBuilder;

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

fn main() -> Result<(), Box<dyn Error>> {
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream_handle.mixer());

    let mf : String = "assets/music.mp3".to_string();
    let tape = get_decoded_mp3(&mf).build_looped()?;    
    let buffer_ms : u64 = get_mp3_duration(get_decoded_mp3(&mf));

    sink.append(tape);

    let num_chunks : u32  = 6;
    let chunk_len : u64 = buffer_ms / num_chunks as u64;
    let mut rng = rand::rng();
    
    loop {
         let mut n : u64 = rng.random_range(0..num_chunks).into();
         n *= chunk_len;
         sink.try_seek(Duration::from_millis(n))?;
         sleep(Duration::from_millis(chunk_len));
    }
       
}
