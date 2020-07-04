extern crate spidev;

use palette::Hsv;
use palette::Srgb;
use palette::encoding::pixel::Pixel;

use spidev::Spidev;
use spidev::SpidevOptions;
use spidev::SpiModeFlags;

use std::cmp;
use std::io;
use std::io::Write;
use std::io::BufWriter;
// use rand::prelude::*;

use std::time::{Instant};

use sysfs_gpio::{Direction, Edge, Pin};

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

fn send_led(w: &mut Write, m: u8, r: u8, g: u8, b: u8) -> io::Result<usize> {
    w.write(&[m, b, g, r])
}

fn main() {
    println!("Starting rusty-wheels");

    let poller = match setup_magnet() {
      Ok(poller) => poller,
      Err(e) => panic!("magnet setup returned an error: {}", e)
    };

    match run_leds(poller) {
      Ok(_) => println!("runleds finished ok"),
      Err(e) => println!("runleds returned an error: {}", e)
    }

    println!("Ending rusty-wheels");
}

fn setup_magnet() -> std::result::Result<sysfs_gpio::PinPoller, sysfs_gpio::Error> {
    let pin = Pin::new(27);
    pin.export()?;
    println!("X1");
    pin.set_direction(Direction::In)?;
    pin.set_edge(Edge::RisingEdge)?;
    let mut poller: sysfs_gpio::PinPoller = pin.get_poller()?;
    println!("X3.1");
    match poller.poll(0)? { 
      Some(value) => println!("POLL {}", value),
      None => ()
    }
    println!("X2");

  Ok(poller)
}

fn run_leds(mut poller: sysfs_gpio::PinPoller) -> io::Result<()> {

    let spi = create_spi()?;

    let mut led_stream = BufWriter::new(spi);

    let start_time = Instant::now();

    let mut spin_start_time = start_time;
    let mut last_spin_start_time = start_time;

    let mut base: f32 = 0.0;

    let num_rainbow = 8;
    let degs_per_led = 360.0 / (num_rainbow as f32);

    // initial blankout
    send_led(&mut led_stream, 0, 0, 0, 0)?;
    for _ in 0..25 {
      send_led(&mut led_stream, 255, 0, 0, 0)?;
    }

    loop {


    match poller.poll(0) { 
      Ok(Some(value)) => {
        println!("Poll got a value {}", value);
        last_spin_start_time = spin_start_time;
        spin_start_time = Instant::now()
      }
      _ => ()
    };

    let now_secs = start_time.elapsed().as_secs();

    let spin_length = spin_start_time - last_spin_start_time;

    let spin_pos = (spin_start_time.elapsed().as_millis() as f32) / (cmp::max(1,spin_length.as_millis()) as f32);

    send_led(&mut led_stream, 0, 0, 0, 0)?;

    for _ in 0..3 {
      if now_secs % 2 == 0 {
        send_led(&mut led_stream, 255, 128, 32, 0)?;
      } else {
        send_led(&mut led_stream, 255, 0, 0, 0)?;
      }
    }

    send_led(&mut led_stream, 255, 0, 0, 0)?;


    for led in 0..num_rainbow {
      // let hue = random::<u8>(); // TODO: needs to go 0..360, not 0..255
  
      // let hue: f32 = (base + degs_per_led * (led as f32)) % 360.0;
      let mut hue = spin_pos * 360.0;

      if hue > 360.0 {
        hue = 360.0;
      }
 
      // println!("hue is {}", hue);
      let hsv: Hsv = Hsv::from_components((hue, 1.0, 0.3));
 
      let srgb = Srgb::from(hsv);
 
      let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

      let [red, green, blue] = pixels;
 
      send_led(&mut led_stream, 255, red, green, blue)?;

      base += 0.008;

    }

    send_led(&mut led_stream, 255, 0, 0, 0)?;


    for _ in 0..3 {
      if now_secs % 2 == 1 {
        send_led(&mut led_stream, 255, 128, 32, 0)?;
      } else {
        send_led(&mut led_stream, 255, 0, 0, 0)?;
      }
    }


    // may need to pad more if many LEDs but this is enough for one side
    // of the wheel
    send_led(&mut led_stream, 0, 0, 0, 0)?;
    led_stream.flush()?;
    }
    let duration_secs = start_time.elapsed().as_secs();

    println!("Duration {} seconds", duration_secs);

    println!("ending");
    Ok(())
}
