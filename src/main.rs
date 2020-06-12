extern crate spidev;

use spidev::Spidev;
use spidev::SpidevOptions;
use spidev::SpiModeFlags;

use std::io;
use std::io::Write;
use std::io::BufWriter;

use std::time::{Instant};

fn create_spi() -> io::Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;

    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(1_000_000)
         .mode(SpiModeFlags::SPI_MODE_0)
         .build();
    spi.configure(&options)?;

    Ok(spi)
}

fn send_frame(w: &mut Write, m: u8, r: u8, g: u8, b: u8) -> io::Result<usize> {
    w.write(&[m, b, g, r])
}

fn main() {
    println!("Starting rusty-wheels");

    match run_leds() {
      Ok(_) => println!("runleds finished ok"),
      Err(e) => println!("runleds returned an error: {}", e)
    }

    println!("Ending rusty-wheels");
}

fn run_leds() -> io::Result<()> {

    let spi = create_spi()?;

    let mut led_stream = BufWriter::new(spi);

    let start_time = Instant::now();

    for _ in 0..9000 {
    send_frame(&mut led_stream, 0, 0, 0, 0)?;

    for _ in 0..8 {
      send_frame(&mut led_stream, 255, 32, 0, 0)?;
      send_frame(&mut led_stream, 255, 0, 32, 0)?;
      send_frame(&mut led_stream, 255, 0, 0, 255)?;
    }

    // may need to pad more if many LEDs but this is enough for one side
    // of the wheel
    send_frame(&mut led_stream, 0, 0, 0, 0)?;
    led_stream.flush()?;
    }
    let duration_secs = start_time.elapsed().as_secs();

    println!("Duration {} seconds", duration_secs);

    println!("ending");
    Ok(())
}
