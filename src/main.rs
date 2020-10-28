mod leds;

use palette::Hsv;
use palette::Srgb;
use palette::encoding::pixel::Pixel;

use signal_hook::flag;

use std::cmp;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use std::time::{Duration, Instant};

use sysfs_gpio::{Direction, Edge, Pin};

use leds::WheelLEDs;

fn main() {
    println!("Starting rusty-wheels");

    let poller = match setup_magnet() {
      Ok(poller) => poller,
      Err(e) => panic!("magnet setup returned an error: {}", e)
    };

    let wheel_leds = WheelLEDs::new();

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

    let mode_phase_0: u64 = (framestate.now.as_secs() / 20) % 6;
    let mode_phase_1: u64 = (framestate.now.as_secs() / 22 + 3) % 6;

    match mode_phase_0 {
        0 => render_side_rainbows(0, wheel_leds, framestate),
        1 => render_side_sliders(0, wheel_leds, framestate),
        2 => render_rgb_trio(0, wheel_leds, framestate),
        3 => render_centre_red(0, wheel_leds, framestate),
        4 => render_rainbow_speckle(0, wheel_leds, framestate),
        5 => render_bitmap(0, wheel_leds, framestate),
        _ => panic!("unknown mode phase 0")
    }?;

    match mode_phase_1 {
        0 => render_side_rainbows(1, wheel_leds, framestate),
        1 => render_side_sliders(1, wheel_leds, framestate),
        2 => render_rgb_trio(1, wheel_leds, framestate),
        3 => render_centre_red(1, wheel_leds, framestate),
        4 => render_rainbow_speckle(1, wheel_leds, framestate),
        5 => render_bitmap(1, wheel_leds, framestate),
        _ => panic!("unknown mode phase 1")
    }?;

    Ok(())
}

/// This renders the first side of the wheel with:
///  * an 8 pixel rainbow around the wheel
///  * a constant blue LED
///  * green and purple LEDs that tick once per frame
///    to show the size of a rotational-pixel
fn render_side_rainbows(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    for led in 0..8 {
      wheel_leds.set(side, led, (0,0,0));
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
      wheel_leds.set(side, led, (red, green, blue));
    }

    wheel_leds.set(side, 16, (0,0,0));

    wheel_leds.set(side, 17, (0,0,255));

    wheel_leds.set(side, 18, (0,0,0));

    let counter_phase  = framestate.loop_counter % 6;
    if counter_phase == 0 {
      wheel_leds.set(side, 19, (0,0,0));
      wheel_leds.set(side, 20, (0,255,0));
      wheel_leds.set(side, 21, (0,0,0));
      wheel_leds.set(side, 22, (0,64,0));
    } else if counter_phase == 3 {
      wheel_leds.set(side, 19, (32, 0, 32));
      wheel_leds.set(side, 20, (0, 0, 0));
      wheel_leds.set(side, 21, (128, 0, 128));
      wheel_leds.set(side, 22, (0, 0, 0));
    } else {
      wheel_leds.set(side, 19, (0, 0, 0));
      wheel_leds.set(side, 20, (0, 0, 0));
      wheel_leds.set(side, 21, (0, 0, 0));
      wheel_leds.set(side, 22, (0, 0, 0));
    }

    Ok(())
}


/// This renders the second side of the wheel two overlaid patterns:
///  * a green time-based line
///  * a magenta spin position line
fn render_side_sliders(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

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
      } else if l == (spin_back_led + 8) % 23 {
        r = 255
      } else if l == (spin_back_led + 16) % 23 {
        r = 255
      } else {
        r = 0
      }

      wheel_leds.set(side, l, (r, g, r));
    }
    Ok(())
    }


/// This renders three slices with black between them, each slice being one
/// of red, green or blue
fn render_rgb_trio(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    for led in 0..23 {

        // led 0 should be dimmest
        // led 22 the brightest
        // this will exponentially scale up to 128 max
        let brightness = 1 << (led / 3);
        let colour: (u8, u8, u8);
        
        if framestate.spin_pos < 0.16 {
            colour = (brightness, 0, 0);
        } else if framestate.spin_pos < 0.32 {
            colour = (0, 0, 0);
        } else if framestate.spin_pos < 0.48 {
            colour = (0, brightness, 0);
        } else if framestate.spin_pos < 0.64 {
            colour = (0, 0, 0);
        } else if framestate.spin_pos < 0.80 {
            colour = (0, 0, brightness);
        } else {
            colour = (0, 0, 0);
        }
   
        wheel_leds.set(side, led, colour);
    }

    Ok(())
}

// renders the middle pixels on each side bright red, with the
// edges (outer and hubwards) fading down to black
fn render_centre_red(side: usize, wheel_leds: &mut WheelLEDs, _framestate: &FrameState) -> io::Result<()> {

    // establish a blank canvas
    for led in 0 .. 23 {
        wheel_leds.set(side, led, (0,0,0));
    }

    for n in 0 .. 8 {
        let colour = (1<<(7-n), 0, 0);
        wheel_leds.set(side, 12 + n, colour);
        wheel_leds.set(side, 11 - n, colour);
    }

    Ok(())
}

fn render_rainbow_speckle(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {
    // establish a blank canvas
    for led in 0 .. 23 {
        wheel_leds.set(side, led, (0,0,0));
    }

    let mut hue = framestate.spin_pos * 360.0;

    if hue > 360.0 {
      hue = 360.0;
    }
 
    let hsv: Hsv = Hsv::from_components((hue, 1.0, 0.2));
 
    let srgb = Srgb::from(hsv);
 
    let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

    let [red, green, blue] = pixels;

    let colour = (red, green, blue);

    let phase = framestate.loop_counter % 4;

    if phase == 0 { 
        for n in 0..6 {
            wheel_leds.set(side, n * 4, colour);
        }
    } else if phase == 2 {
        for n in 0..6 {
            wheel_leds.set(side, n * 4 + 2, colour);
        }
    } 
    // otherwise don't set any pixels

    Ok(())
}


fn render_bitmap(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    // establish a blank canvas
    for led in 0 .. 23 {
        wheel_leds.set(side, led, (0,0,0));
    }

    // render approx 50 pixels in half the rotation
    // or 100 pixels per full rotation
    let row: [u128; 7] = [
        0b011111010001001110010001000000011110001110011110001110001110,
        0b010000010001010001010010000000001001010001010001000100010001,
        0b010000010001010000010100000000001001010001010001000100010000,
        0b011110010001010000011000000000001110010001011110000100001110,
        0b010000010001010000010100000000001001010001010100000100000001,
        0b010000010001010001010010000000001001010001010010000100010001,
        0b010000001110001110010001000000011110001110010001001110001110
        ];

    let mut pixel;

    // rotate the text round the wheel once per minute
    let spin_adj: f32 = ((framestate.now.as_secs() % 60) as f32) / 60.0;

    pixel = (((framestate.spin_pos + spin_adj) % 1.0) * 128.0) as u8;

    // if spin pos too high, maybe we'll go over a limit

    if pixel > 127 {
        pixel = 127;
    }

    // flip pixels on other side because rotation is the other way round
    if side == 1 {
        pixel = 127-pixel;
    }

    for n in 0..7 {
        let r: u8 = ((((row[n] & (1 << pixel)) >> pixel) & 1) << 7) as u8;
        let colour = match side {
            0 => (r, 0, r), // magenta
            1 => (0, r, r), // cyan
            _ => panic!("impossible side number")
        };
        wheel_leds.set(side, 22-n, colour);    
    }

    Ok(())
}

