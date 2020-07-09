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

    // initial blankout
    send_led(&mut led_stream, 0, 0, 0, 0)?;
    for _ in 0..25 {
      send_led(&mut led_stream, 255, 0, 0, 0)?;
    }

    let mut loop_counter: u32 = 0;

    loop {


    match poller.poll(0) { 
      Ok(Some(value)) => {
        println!("Poll got a value {}", value);
        last_spin_start_time = spin_start_time;
        spin_start_time = Instant::now()
      }
      _ => ()
    };

    let now_millis = start_time.elapsed().as_millis();
    let now_secs = start_time.elapsed().as_secs();

    let spin_length = spin_start_time - last_spin_start_time;

    // initialise LED stream
    send_led(&mut led_stream, 0, 0, 0, 0)?;

    let mode_duration = cmp::max(spin_start_time.elapsed(), spin_length);

    if mode_duration.as_millis() > 2000 || mode_duration.as_millis() == 0 { 
      // stopped mode
      let flicker = (now_millis / 25) % 4 == 0;
      let topside = now_secs % 2 == 0;
      for side in 0..2 {
        for led in 0..6 {
          send_led(&mut led_stream, 255, 0, 0, 0)?;
        }
        for led in 6..8 {
          send_led(&mut led_stream, 255, 128, 0, 0)?;
        }
        for led in 8..10 {
          send_led(&mut led_stream, 255, 0, 0, 0)?;
        }
        if topside ^ (side == 0){
          for led in 10..13 {
            if flicker {
              send_led(&mut led_stream, 255, 128, 64, 0)?;
            } else { 
             send_led(&mut led_stream, 255, 0, 0, 0)?;
            }
          }

        } else {
          for led in 10..13 {
            send_led(&mut led_stream, 255, 0, 0, 0)?;
          }
        }
        for led in 13..15 {
          send_led(&mut led_stream, 255, 0, 0, 0)?;
        }
        for led in 15..17 {
          send_led(&mut led_stream, 255, 128, 0, 0)?;
        }
        for led in 17..23 {
          send_led(&mut led_stream, 255, 0, 0, 0)?;
        }
      }


    } else {

    let spin_pos = (spin_start_time.elapsed().as_millis() as f32) / (cmp::max(1,spin_length.as_millis()) as f32);


    for led in 0..8 {
      send_led(&mut led_stream, 255, 0, 0, 0)?;
    }

    for led in 8..16 {
      // let hue = random::<u8>(); // TODO: needs to go 0..360, not 0..255
  
      let mut hue = spin_pos * 360.0;

      if hue > 360.0 {
        hue = 360.0;
      }
 
      // println!("hue is {}", hue);
      let hsv: Hsv = Hsv::from_components((hue, 1.0, 0.2));
 
      let srgb = Srgb::from(hsv);
 
      let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

      let [red, green, blue] = pixels;
 
      send_led(&mut led_stream, 255, red, green, blue)?;

      base += 0.008;

    }

    send_led(&mut led_stream, 255, 0, 0, 0)?;

    send_led(&mut led_stream, 255, 0, 0, 255)?; // permanently on

    send_led(&mut led_stream, 255, 0, 0, 0)?;

    let counter_phase  = loop_counter % 6;
    if counter_phase == 0 {
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 0, 255, 0)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 0, 64, 0)?;
    } else if counter_phase == 3 {
      send_led(&mut led_stream, 255, 32, 0, 32)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 128, 0, 128)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
    } else {
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
      send_led(&mut led_stream, 255, 0, 0, 0)?;
    }

    // this should range from 0..23 over the period of 1 second, which is
    // around the right time for one wheel spin
    let back_led: u32 = ((now_millis % 1000) * 23 / 1000) as u32;

    let spin_back_led: u32 = (spin_pos * 23.0) as u32;

    for l in 0..23 {
      let g;
      if l == back_led {
        g = 255
      } else {
        g = 0
      }
      let r;
      if l == spin_back_led {
        r = 255
      } else {
        r = 0
      }
      send_led(&mut led_stream, 255, r, g, r)?;
    }

    // may need to pad more if many LEDs but this is enough for one side
    // of the wheel
    send_led(&mut led_stream, 0, 0, 0, 0)?;
    send_led(&mut led_stream, 0, 0, 0, 0)?;
    send_led(&mut led_stream, 0, 0, 0, 0)?;

    }

    led_stream.flush()?;
    loop_counter = loop_counter + 1;
    }
    let duration_secs = start_time.elapsed().as_secs();

    println!("Duration {} seconds", duration_secs);

    println!("ending");
    Ok(())
}
