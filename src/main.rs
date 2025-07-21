use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;

use rand::Rng;

use rodio::OutputStreamBuilder;
use rodio::Sink;
use rodio::Decoder;


fn main() -> Result<(), Box<dyn Error>> {
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream_handle.mixer());

    let file = File::open("assets/music.mp3")?;
    sink.append(Decoder::try_from(file)?);

    let mut rng = rand::rng();
    let incr = 100;
    
    loop {
        let mut n = rng.random_range(0..10);
        n *= incr;
        sink.try_seek(Duration::from_millis(n))?;
        sleep(Duration::from_millis(incr));
    }
       
}
