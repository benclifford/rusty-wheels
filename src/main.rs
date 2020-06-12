extern crate spidev;

use spidev::Spidev;
use spidev::SpidevOptions;
use spidev::SpiModeFlags;

use std::io;
use std::io::Write;

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

fn send_frame(spi: &mut Spidev, m: u8, r: u8, g: u8, b: u8) -> io::Result<usize> {
    spi.write(&[m, b, g, r])
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

    let mut spi = create_spi()?;

    send_frame(&mut spi, 0, 0, 0, 0)?;

    for _ in 0..8 {
      send_frame(&mut spi, 255, 32, 0, 0)?;
      send_frame(&mut spi, 255, 0, 32, 0)?;
      send_frame(&mut spi, 255, 0, 0, 255)?;
    }

    // may need to pad more if many LEDs but this is enough for one side
    // of the wheel
    send_frame(&mut spi, 0, 0, 0, 0)?;

    println!("ending");
    Ok(())
}
