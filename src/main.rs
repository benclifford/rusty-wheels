mod leds;
mod magnet;
mod buttons;

use palette::encoding::pixel::Pixel;
use palette::Hsv;
use palette::Srgb;

use signal_hook::flag;

use std::cmp;
use std::f32::consts::TAU;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use std::time::{Duration, Instant};

use rand::Rng;

use leds::WheelLEDs;
use magnet::Magnet;

use buttons::PushButton;

/// The duration between magnet pulses that distinguishes between
/// stopped mode and live mode.
const STOP_TIME_MS: u128 = 2000;

fn main() {
    println!("Starting rusty-wheels");

    let magnet = match Magnet::new() {
        Ok(m) => m,
        Err(e) => panic!("magnet setup returned an error: {}", e),
    };

    let push_button = match PushButton::new() {
        Ok(m) => m,
        Err(e) => panic!("push button setup returned an error: {}", e),
    };

    let wheel_leds = WheelLEDs::new();

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    match run_leds(magnet, wheel_leds, push_button, shutdown_flag) {
        Ok(_) => println!("runleds finished ok"),
        Err(e) => println!("runleds returned an error: {}", e),
    }

    println!("Ending rusty-wheels");
}

fn run_leds(
    mut m: Magnet,
    mut wheel_leds: WheelLEDs,
    mut push_button: PushButton,
    shutdown_flag: Arc<AtomicBool>,
) -> io::Result<()> {
    let start_time = Instant::now();

    let mut spin_start_time = start_time;
    let mut last_spin_start_time = start_time;

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    }
    wheel_leds.show()?;

    let mut loop_counter: u32 = 0;

    flag::register(signal_hook::SIGTERM, Arc::clone(&shutdown_flag))?;
    flag::register(signal_hook::SIGINT, Arc::clone(&shutdown_flag))?;

    let mut p: bool = true;

    // this should perhaps be a "new mode" time - doesn't need to be a cycling mode number
    // as all that is captured in the "mode" reference.
    let mut mode_phase: usize = MODES.len() + 1; // pick a mode value that will be trigger new mode initialisation immediately

    // this is going to get replaced pretty much right away unless I implement a count-down timer mode switcher rather than
    // absolute time based phasing. But it's better than threading Option behaviour all the way through.
    let mut mode: Box<dyn Mode> = MODES[0]();

    while !(shutdown_flag.load(Ordering::Relaxed)) {
        if m.pulsed() {
            last_spin_start_time = spin_start_time;
            spin_start_time = Instant::now()
        };

        if push_button.pulsed() {
            println!("push button pulse");
            p = !p;
        }

        let spin_length = spin_start_time - last_spin_start_time;

        let mode_duration = cmp::max(spin_start_time.elapsed(), spin_length);

        let framestate = FrameState {
            now: start_time.elapsed(),
            loop_counter: loop_counter,
            spin_pos: (spin_start_time.elapsed().as_millis() as f32)
                / (cmp::max(1, spin_length.as_millis()) as f32),
        };

        if mode_duration.as_millis() > STOP_TIME_MS || mode_duration.as_millis() == 0 {
            if p {
                render_stopped_mode(&mut wheel_leds, &framestate)?;
            } else {
                render_other_stopped_mode(&mut wheel_leds, &framestate)?;
            }
        } else {

            let next_mode_phase: usize = ((framestate.now.as_secs() / 20) % (MODES.len() as u64)) as usize;

            if next_mode_phase != mode_phase {
                mode_phase = next_mode_phase;
                mode = MODES[mode_phase]();
            }

            mode.render(0, &mut wheel_leds, &framestate)?;
            mode.render(1, &mut wheel_leds, &framestate)?;
        }

        wheel_leds.show()?;

        loop_counter += 1;
    }
    let duration_secs = start_time.elapsed().as_secs();
    println!("Duration {} seconds", duration_secs);

    // run a shutdown effect

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (1, 1, 1));
        }
    }
    wheel_leds.show()?;

    thread::sleep(Duration::from_millis(250));

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (0, 0, 0));
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
    spin_pos: f32,
}

fn render_other_stopped_mode(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {

    for side in 0..2 {
        for led in 0..23 {
            wheel_leds.set(side, led, (32, 32, 32));
        }
        // override the middle ones with full brightness
        for led in 9..14 {
            wheel_leds.set(side, led, (255, 255, 255));
        }
    }

    Ok(())
}

fn render_stopped_mode(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {
    let now_millis = framestate.now.as_millis();
    let now_secs = framestate.now.as_secs();
    let flicker = (now_millis / 25) % 4 == 0;
    let topside = now_secs % 2 == 0;
    for side in 0..2 {
        for led in 0..2 {
            wheel_leds.set(side, led, (2, 0, 0));
        }
        for led in 2..4 {
            wheel_leds.set(side, led, (8, 0, 0));
        }
        for led in 4..6 {
            wheel_leds.set(side, led, (64, 0, 0));
        }

        for led in 6..8 {
            wheel_leds.set(side, led, (255, 0, 0));
        }

        for led in 8..9 {
            wheel_leds.set(side, led, (0, 0, 0));
        }

        if topside ^ (side == 0) {
            for led in 9..14 {
                if flicker {
                    wheel_leds.set(side, led, (255, 255, 0));
                } else {
                    wheel_leds.set(side, led, (0, 0, 0));
                }
            }
        } else {
            for led in 9..14 {
                wheel_leds.set(side, led, (0, 0, 0));
            }
        }
        for led in 14..15 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
        for led in 15..17 {
            wheel_leds.set(side, led, (255, 0, 0));
        }
        for led in 17..19 {
            wheel_leds.set(side, led, (64, 0, 0));
        }
        for led in 19..21 {
            wheel_leds.set(side, led, (8, 0, 0));
        }
        for led in 21..23 {
            wheel_leds.set(side, led, (2, 0, 0));
        }
    }

    Ok(())
}


trait Mode {
    fn render(&self, side: usize, leds: &mut leds::WheelLEDs, frame: &FrameState) -> io::Result<()>;
}

struct StatelessMode {
    render_fn: fn(side: usize, leds: &mut leds::WheelLEDs, frame: &FrameState) -> io::Result<()>
}

impl Mode for StatelessMode {
    fn render(&self, side: usize, leds: &mut leds::WheelLEDs, frame: &FrameState) -> io::Result<()> {
        (self.render_fn)(side, leds, frame)
    }
}

macro_rules! stateless_mode {
  ( $x:expr ) => { || 
      Box::new(StatelessMode {
                    render_fn: $x
                })
   }
}


const MODES: &[fn() -> Box<dyn Mode>] = &[
    stateless_mode!(render_fade_quarters),
    stateless_mode!(render_random_rim),
    stateless_mode!(render_helix),
    stateless_mode!(render_europa),
    stateless_mode!(render_pulsed_rainbow),
    stateless_mode!(render_rainbow_rim),
    stateless_mode!(render_fade_spirals),
    stateless_mode!(render_radial_stripes),
    stateless_mode!(render_graycode_rim),
    stateless_mode!(render_sine_full),
    stateless_mode!(render_sine),
    stateless_mode!(render_mod_speckle),
    stateless_mode!(render_speckle_onepix),
    stateless_mode!(render_speckle_random),
    stateless_mode!(render_rainbows),
    stateless_mode!(render_sliders),
    stateless_mode!(render_rgb_trio),
    stateless_mode!(render_centre_red),
    stateless_mode!(render_rainbow_speckle),
    stateless_mode!(render_bitmap),
    stateless_mode!(render_phrase),
];


/// This renders the first side of the wheel with:
///  * an 8 pixel rainbow around the wheel
///  * a constant blue LED
///  * green and purple LEDs that tick once per frame
///    to show the size of a rotational-pixel
fn render_rainbows(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..8 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let rainbow_colour = spinpos_to_rgb(framestate);

    for led in 8..16 {
        wheel_leds.set(side, led, rainbow_colour);
    }

    wheel_leds.set(side, 16, (0, 0, 0));

    wheel_leds.set(side, 17, (0, 0, 255));

    wheel_leds.set(side, 18, (0, 0, 0));

    let counter_phase = framestate.loop_counter % 6;
    if counter_phase == 0 {
        wheel_leds.set(side, 19, (0, 0, 0));
        wheel_leds.set(side, 20, (0, 255, 0));
        wheel_leds.set(side, 21, (0, 0, 0));
        wheel_leds.set(side, 22, (0, 64, 0));
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

/// This renders the first side of the wheel with
/// an 8 pixel rainbow around the rim of wheel
fn render_rainbow_rim(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let rainbow_colour = spinpos_to_rgb(framestate);

    for led in 15..23 {
        wheel_leds.set(side, led, rainbow_colour);
    }

    Ok(())
}

fn render_random_rim(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..20 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    // starting at 1 avoids having all three bits off
    // (the 0 position) so there will always at least
    // be one LED on in each frame
    let n = rand::thread_rng().gen_range(1, 8);

    for led in 0..3 {
        if n & (1 << led) != 0 {
            wheel_leds.set(side, 20 + led, (255, 0, 0));
        } else {
            wheel_leds.set(side, 20 + led, (0, 0, 0));
        }
    }

    Ok(())
}

fn render_pulsed_rainbow(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..15 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    for led in 15..23 {
        let led_n = led - 15;
        let frac: f32 = ((led - 15) as f32) / 8.0;
        let v1 = (framestate.spin_pos + frac) % 1.0;
        let v2 = (v1 * (led_n as f32 + 2.0)) % 1.0;
        let v3 = if v2 > 0.5 { 1.0 } else { 0.0 };
        let rainbow_colour = fraction_to_rgb(frac, Some(v3));
        wheel_leds.set(side, led, rainbow_colour);
    }

    Ok(())
}

/// This renders the second side of the wheel two overlaid patterns:
///  * a green time-based line
///  * a magenta spin position line
fn render_sliders(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
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
fn render_rgb_trio(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
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
fn render_centre_red(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    for n in 0..8 {
        let colour = (1 << (7 - n), 0, 0);
        wheel_leds.set(side, 12 + n, colour);
        wheel_leds.set(side, 11 - n, colour);
    }

    Ok(())
}

/// Turns spin position into a saturated rainbow wheel
fn spinpos_to_rgb(framestate: &FrameState) -> (u8, u8, u8) {
    fraction_to_rgb(framestate.spin_pos, None)
}

/// turns a value from 0..1 into RGB
fn fraction_to_rgb(fraction: f32, value: Option<f32>) -> (u8, u8, u8) {
    let hue = (fraction * 360.0).min(360.0);

    let real_value = match value {
        Some(v) => v,
        None => 0.2,
    };

    let hsv: Hsv = Hsv::from_components((hue, 1.0, real_value));

    let srgb = Srgb::from(hsv);

    let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

    let [red, green, blue] = pixels;

    (red, green, blue)
}

fn render_rainbow_speckle(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let colour = spinpos_to_rgb(framestate);

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

fn render_mod_speckle(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        let m = framestate.loop_counter % (2 + (22 - led) as u32);
        if m == 0 {
            wheel_leds.set(side, led, (255, 255, 0));
        } else {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    }

    Ok(())
}

fn render_speckle_onepix(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    let mut done = false;
    for led in 0..23 {
        let m = framestate.loop_counter % (2 + (22 - led) as u32);
        if m == 0 && !done {
            wheel_leds.set(side, led, (255, 255, 0));
            done = true;
        } else {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    }

    Ok(())
}

fn render_speckle_random(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }
    let rand_led = rand::thread_rng().gen_range(0, 23);
    let rand_rgb = rand::thread_rng().gen_range(0, 3);
    let colour = match rand_rgb {
        0 => (255, 0, 0),
        1 => (0, 255, 0),
        2 => (0, 0, 255),
        _ => (1, 1, 1), // shouldn't happen with choice of rand_rgb
    };
    wheel_leds.set(side, rand_led, colour);

    Ok(())
}

fn render_bitmap(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // render approx 50 pixels in half the rotation
    // or 100 pixels per full rotation
    let row: [u128; 7] = [
        0b01111101000100111001000100000001111000111001111000111000111000000,
        0b01000001000101000101001000000000100101000101000100010001000100000,
        0b01000001000101000001010000000000100101000101000100010001000000000,
        0b01111001000101000001100000000000111001000101111000010000111000000,
        0b01000001000101000001010000000000100101000101010000010000000100000,
        0b01000001000101000101001000000000100101000101001000010001000100000,
        0b01000000111000111001000100000001111000111001000100111000111000000,
    ];

    helper_render_bitmap(&row, side, wheel_leds, framestate)
}

// These should become part of mode-local state object that isn't
// implemented at the moment, using an as-yet-undefined RenderMode trait
static mut phrase_row: [u128; 7] = [0, 0, 0, 0, 0, 0, 0]; // blank canvas
                                                          // mode-local state wouldn't need an initialized variable: the initialization
                                                          // would initialise the bitmap so it would never be uninitialised before
                                                          // calling render.
static mut phrase_row_initialized: bool = false;

fn render_phrase(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // there are no other threads around to mess with this, so
    // this unsafe should be allowed. In the presence of per-mode
    // state objects, this unsafe should go away entirely, replaced
    // by an initialized-at-creation mode state object.
    unsafe {
        if !phrase_row_initialized {
            println!("Iniializing phrase bitmap");
            let phrase = "@BENCLIFFORD";

            let font = bdf::open("/home/pi/src/rusty-wheels/font.bdf").expect("Valid font");

            for c in phrase.chars() {
                let glyph = font
                    .glyphs()
                    .get(&c)
                    .unwrap_or_else(|| panic!("Cannot get glyph"));

                // first shift the existing display rightwards by the appropriate number of pixels
                let width = glyph.width() + 1;
                for row in 0..7 {
                    // assume 7 rows here
                    phrase_row[row] <<= width;
                }

                // now render the glyph
                for row in (0 as usize)..7 {
                    for col in 0..glyph.width() {
                        if glyph.get(col, row as u32) {
                            phrase_row[row] = phrase_row[row] | 1 << (glyph.width() - col - 1);
                        }
                    }
                }
            }

            phrase_row_initialized = true;
        }

        // see above note  - this happens because phrase_row is being passed in
        helper_render_bitmap(&phrase_row, side, wheel_leds, framestate)
    }
}

/// This can render a 128 pixel wide, 7 bit pixel high bitmap
fn helper_render_bitmap(
    row: &[u128; 7],
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }
    let mut pixel;

    // can be used to adjust where the bitmap starts rendering
    // but has different physical meaning on each side
    const spin_adj: f32 = 0.0;

    pixel = (((framestate.spin_pos + spin_adj) % 1.0) * 128.0) as u8;

    // if spin pos too high, maybe we'll go over a limit

    if pixel > 127 {
        pixel = 127;
    }

    // flip pixels on other side because rotation is the other way round
    if side == 1 {
        pixel = 127 - pixel;
    }

    for n in 0..7 {
        let r = ((row[n] & (1 << pixel)) >> pixel) & 1;
        let colour = if r != 0 {
            match side {
                0 => (255, 32, 0), // amber
                1 => (56, 255, 0), // green - from wikipedia phosper wavelength converted to rgb
                _ => panic!("impossible side number"),
            }
        } else {
            (0, 0, 0)
        };
        wheel_leds.set(side, 22 - n, colour);
    }

    Ok(())
}

fn render_sine(side: usize, wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = (17.0 + phase * 5.0) as usize;

    wheel_leds.set(side, led, (0, 255, 0));

    Ok(())
}

fn render_helix(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = cmp::min((17.0 + phase * 6.0) as usize, 22);
    wheel_leds.set(side, led, (64, 0, 64));

    let led = cmp::min((17.0 - phase * 6.0) as usize, 22);
    wheel_leds.set(side, led, (0, 255, 0));

    Ok(())
}

fn render_sine_full(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let phase = (framestate.spin_pos * TAU * 10.0).sin();

    // beware of casting to unsigned when there could still be
    // negatives around
    let led = (17.0 + phase * 5.0) as usize;

    wheel_leds.set(side, led, (0, 255, 0));

    let phase2 = (framestate.spin_pos * TAU * 7.0).sin();
    let led2 = (8.0 + phase2 * 3.0) as usize;
    wheel_leds.set(side, led2, (255, 0, 0));

    let phase3 = (framestate.spin_pos * TAU * 3.0).sin();
    let led3 = (3.0 + phase3 * 2.0) as usize;
    wheel_leds.set(side, led3, (0, 0, 255));

    Ok(())
}

fn render_graycode_rim(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let segment = (framestate.spin_pos * 8.0) as u8; // could go over 8 because spinpos can go over 1

    let gray = segment ^ (segment >> 1);

    let amber = (255, 32, 0);

    if (gray & 0b001) != 0 {
        wheel_leds.set(side, 22, amber);
        wheel_leds.set(side, 21, amber);
        wheel_leds.set(side, 20, amber);
    }

    if (gray & 0b010) != 0 {
        wheel_leds.set(side, 19, amber);
        wheel_leds.set(side, 18, amber);
        wheel_leds.set(side, 17, amber);
    }

    if (gray & 0b100) != 0 {
        wheel_leds.set(side, 16, amber);
        wheel_leds.set(side, 15, amber);
        wheel_leds.set(side, 14, amber);
    }

    Ok(())
}

fn render_radial_stripes(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let segment = (framestate.spin_pos * 32.0) as u8; // could go over 32 because spinpos can go over 1

    if segment % 2 == 1 {
        for led in 12..23 {
            wheel_leds.set(side, led, (64, 64, 64));
        }
    }

    Ok(())
}

fn render_europa(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blue canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 32));
    }

    let segment = (framestate.spin_pos * 12.0) % 1.0; // could go over 12 because spinpos can go over 1

    if segment < 0.08 || (segment >= 0.16 && segment < 0.24) {
        wheel_leds.set(side, 18, (255, 255, 0));
    } else if segment < 0.16 {
        for led in 17..20 {
            wheel_leds.set(side, led, (255, 255, 0));
        }
    }

    Ok(())
}

fn render_fade_spirals(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas

    let s1 = cmp::min(22, (23.0 * framestate.spin_pos) as i8);
    let s2 = cmp::min(22, (23.0 * ((framestate.spin_pos + 0.5) % 1.0)) as i8);

    for led in 0..23 {
        let dist_s1 = (s1 - led).abs() as u8;
        let dist_s2 = (s2 - led).abs() as u8;
        if dist_s1 < dist_s2 {
            if dist_s1 > 7 {
                wheel_leds.set(side, led as usize, (0, 0, 0));
            } else {
                let v = (2 as u8).pow((7 - dist_s1) as u32);
                wheel_leds.set(side, led as usize, (0, v, 0));
            }
        } else {
            if dist_s2 > 7 {
                wheel_leds.set(side, led as usize, (0, 0, 0));
            } else {
                let v = (2 as u8).pow((7 - dist_s2) as u32);
                wheel_leds.set(side, led as usize, (v, 0, v));
            }
        }
    }

    Ok(())
}

fn render_fade_quarters(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {

    let fade_frac = (framestate.spin_pos * 4.0) % 1.0;

    // some gamma correction
    let brightness = fade_frac.powf(3.0).min(1.0);

    let pix_brightness_red = (255.0 * brightness) as u8;
    let pix_brightness_green = (64.0 * brightness) as u8;

    for led in 0..11 {
        wheel_leds.set(side, led as usize, (0, 0, 0));
    }
    for led in 11..23 {
        wheel_leds.set(side, led as usize, (pix_brightness_red, pix_brightness_green, 0));
    }

    Ok(())
}
