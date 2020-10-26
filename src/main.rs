extern crate spidev;

use palette::Hsv;
use palette::Srgb;
use palette::encoding::pixel::Pixel;

use signal_hook::flag;

use spidev::Spidev;
use spidev::SpidevOptions;
use spidev::SpiModeFlags;

use std::cmp;
use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use std::time::{Duration, Instant};

use sysfs_gpio::{Direction, Edge, Pin};

fn setup_leds() -> io::Result<BufWriter<Spidev>> {
    println!("Configuring LEDs");
    let spi = create_spi()?;
    Ok(BufWriter::new(spi))
}

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

fn send_rgb(w: &mut Write, rgb: (u8, u8, u8)) -> io::Result<usize> {
    let (r, g, b) = rgb;
    send_led(w, 255, r, g, b)
}

fn main() {
    println!("Starting rusty-wheels");

    let poller = match setup_magnet() {
      Ok(poller) => poller,
      Err(e) => panic!("magnet setup returned an error: {}", e)
    };

    let led_stream = match setup_leds() {
      Ok(leds) => leds,
      Err(e) => panic!("LED setup returned an error: {}", e)
    };

    let wheel_leds = WheelLEDs {
        led_stream: led_stream,
        leds: [(0,0,0); 46]
    };

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    match run_leds(poller, wheel_leds, shutdown_flag) {
      Ok(_) => println!("runleds finished ok"),
      Err(e) => println!("runleds returned an error: {}", e)
    }

    println!("Ending rusty-wheels");
}

fn setup_magnet() -> std::result::Result<sysfs_gpio::PinPoller, sysfs_gpio::Error> {
    println!("Configuring magnet");
    let pin = Pin::new(27);
    pin.export()?;
    pin.set_direction(Direction::In)?;
    pin.set_edge(Edge::RisingEdge)?;
    let mut poller: sysfs_gpio::PinPoller = pin.get_poller()?;
    println!("Making first pin poll");
    match poller.poll(0)? { 
      Some(value) => println!("Poll got first value {} - ignoring", value),
      None => ()
    }
    println!("Done configuring magnet");

  Ok(poller)
}

fn run_leds(mut poller: sysfs_gpio::PinPoller, mut wheel_leds: WheelLEDs, shutdown_flag: Arc<AtomicBool>) -> io::Result<()> {

    let start_time = Instant::now();

    let mut spin_start_time = start_time;
    let mut last_spin_start_time = start_time;

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (0,0,0));
        }
    }
    wheel_leds.show()?;

    let mut loop_counter: u32 = 0;

    flag::register(signal_hook::SIGTERM, Arc::clone(&shutdown_flag))?;
    flag::register(signal_hook::SIGINT, Arc::clone(&shutdown_flag))?;

    while !(shutdown_flag.load(Ordering::Relaxed)) {

    match poller.poll(0) { 
      Ok(Some(value)) => {
        println!("Poll got a value {}", value);
        last_spin_start_time = spin_start_time;
        spin_start_time = Instant::now()
      }
      _ => ()
    };

    let spin_length = spin_start_time - last_spin_start_time;

    let mode_duration = cmp::max(spin_start_time.elapsed(), spin_length);

    let framestate = FrameState {
        now: start_time.elapsed(),
        loop_counter: loop_counter,
        spin_pos: (spin_start_time.elapsed().as_millis() as f32) / (cmp::max(1,spin_length.as_millis()) as f32),
    };

    if mode_duration.as_millis() > 2000 || mode_duration.as_millis() == 0 {
      render_stopped_mode(&mut wheel_leds, &framestate)?;
    } else {
      render_live_mode(&mut wheel_leds, &framestate)?;
    }

    wheel_leds.show()?;

    loop_counter = loop_counter + 1;
    }
    let duration_secs = start_time.elapsed().as_secs();
    println!("Duration {} seconds", duration_secs);

    // run a shutdown effect

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (1,1,1));
        }
    }
    wheel_leds.show()?;

    thread::sleep(Duration::from_millis(250));

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (0,0,0));
        }
    }
    wheel_leds.show()?;

    println!("ending");
    Ok(())
}

/// A FrameState contains information about the position and timing of
/// the bike wheel useful for rendering a frame.
struct FrameState {
  /// Duration since the executable started
  now: Duration,

  /// A count of the number of frames rendered. This will increase by one
  /// on each render, regardless of time or wheel rotation.
  loop_counter: u32,

  /// An estimate of the current position of the wheel, ranging from 0 to
  /// approximately 1. This might go above 1 if the bike is slowing down,
  /// so code needs to accept that.
  spin_pos: f32
}


/// WheelLEDs provides some kind of array-like access to setting individual
/// LEDs which can then be dumped out in one frame.
/// It provides a mutable collection of RGB tuples, one entry for each LED,
/// structure in two dimensions by radial position and side
/// and a way to dump that array onto the physical LED array.

struct WheelLEDs {
    led_stream: BufWriter<Spidev>,

    /// The LEDs are stored in this array in the order that they should
    /// be sent down the SPI channel.
    leds: [(u8, u8, u8); 46]
}

impl WheelLEDs {

    /// set a pixel, side 0 or 1, pixel 0 ... 22
    /// pixel number starts at the centre of the wheel, on both
    /// sides.
    fn set(&mut self, side: usize, pixel: usize, rgb: (u8, u8, u8)) {
        if side == 0 {
            self.leds[pixel] = rgb;
        } else {
            self.leds[23 + (22-pixel)] = rgb
        }
    }

    /// Writes the stored LED values to the physical strip over SPI
    fn show(&mut self) -> io::Result<()> {

        // initialise LED strip to recieve values from the start
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;

        for led in 0..46 {
          send_rgb(&mut self.led_stream, self.leds[led])?;
        }

        // padding for clocking purposes down-strip
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;
        send_led(&mut self.led_stream, 0, 0, 0, 0)?;

        self.led_stream.flush()?;

        Ok(())
    }
}

fn render_stopped_mode(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {
      let now_millis = framestate.now.as_millis();
      let now_secs = framestate.now.as_secs();
      let flicker = (now_millis / 25) % 4 == 0;
      let topside = now_secs % 2 == 0;
      for side in 0..2 {

        for led in 0..6 {
          wheel_leds.set(side, led, (8, 0, 0));
        }

        for led in 6..8 {
          wheel_leds.set(side, led, (128, 0, 0));
        }

        for led in 8..10 {
          wheel_leds.set(side, led, (0, 0, 0));
        }

        if topside ^ (side == 0){
          for led in 10..13 {
            if flicker {
              wheel_leds.set(side, led, (255, 255, 0));
            } else { 
              wheel_leds.set(side, led, (0, 0, 0));
            }
          }

        } else {
          for led in 10..13 {
            wheel_leds.set(side, led, (0, 0, 0));
          }
        }
        for led in 13..15 {
          wheel_leds.set(side, led, (0, 0, 0));
        }
        for led in 15..17 {
          wheel_leds.set(side, led, (128, 0, 0));
        }
        for led in 17..23 {
          wheel_leds.set(side, led, (8, 0, 0));
        }
      }

    Ok(())
}


fn render_live_mode(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    render_side_1(wheel_leds, framestate)?;
    render_side_2(wheel_leds, framestate)?;

    Ok(())
}

/// This renders the first side of the wheel with:
///  * an 8 pixel rainbow around the wheel
///  * a constant blue LED
///  * green and purple LEDs that tick once per frame
///    to show the size of a rotational-pixel
fn render_side_1(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    for led in 0..8 {
      wheel_leds.set(0, led, (0,0,0));
    }

    let mut hue = framestate.spin_pos * 360.0;

    if hue > 360.0 {
      hue = 360.0;
    }
 
    let hsv: Hsv = Hsv::from_components((hue, 1.0, 0.2));
 
    let srgb = Srgb::from(hsv);
 
    let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

    let [red, green, blue] = pixels;
 
    for led in 8..16 {
      wheel_leds.set(0, led, (red, green, blue));
    }

    wheel_leds.set(0, 16, (0,0,0));

    wheel_leds.set(0, 17, (0,0,255));

    wheel_leds.set(0, 18, (0,0,0));

    let counter_phase  = framestate.loop_counter % 6;
    if counter_phase == 0 {
      wheel_leds.set(0, 19, (0,0,0));
      wheel_leds.set(0, 20, (0,255,0));
      wheel_leds.set(0, 21, (0,0,0));
      wheel_leds.set(0, 22, (0,64,0));
    } else if counter_phase == 3 {
      wheel_leds.set(0, 19, (32, 0, 32));
      wheel_leds.set(0, 20, (0, 0, 0));
      wheel_leds.set(0, 21, (128, 0, 128));
      wheel_leds.set(0, 22, (0, 0, 0));
    } else {
      wheel_leds.set(0, 19, (0, 0, 0));
      wheel_leds.set(0, 20, (0, 0, 0));
      wheel_leds.set(0, 21, (0, 0, 0));
      wheel_leds.set(0, 22, (0, 0, 0));
    }

    Ok(())
}


/// This renders the second side of the wheel two overlaid patterns:
///  * a green time-based line
///  * a magenta spin position line
fn render_side_2(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    let now_millis = framestate.now.as_millis();

    // this should range from 0..23 over the period of 1 second, which is
    // around the right time for one wheel spin
    let back_led: usize = ((now_millis % 1000) * 23 / 1000) as usize;

    let spin_back_led: usize = (framestate.spin_pos * 23.0) as usize;

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
      wheel_leds.set(1, l, (r, g, r));
    }
    Ok(())
    }



