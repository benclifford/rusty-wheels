mod buttons;
mod helpers;
mod leds;
mod magnet;
mod mode_bitmap_text;
mod mode_cellular;
mod mode_edge_strobe;
mod mode_rainbow;
mod mode_speckles;
mod structs;

use signal_hook::flag;

use std::cmp;
use std::f32::consts::TAU;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use std::time::{Duration, Instant};

use rand::Rng;

use helpers::{fraction_to_rgb, spinpos_to_rgb};
use leds::WheelLEDs;
use magnet::Magnet;
use structs::{FrameState, Mode};

use buttons::PushButton;

/// The duration between magnet pulses that distinguishes between
/// stopped mode and live mode.
const STOP_TIME_MS: u128 = 2000;

/// The duration between mode changes.
const MODE_CHANGE_SEC: u64 = 20;

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

    let mut next_mode_time = Instant::now();

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
            spin_length: spin_length,
        };

        if mode_duration.as_millis() > STOP_TIME_MS || mode_duration.as_millis() == 0 {
            if p {
                render_stopped_mode(&mut wheel_leds, &framestate)?;
            } else {
                render_other_stopped_mode(&mut wheel_leds, &framestate)?;
            }
        } else {
            if next_mode_time <= Instant::now() {
                let next_mode = rand::thread_rng().gen_range(0, MODES.len());
                mode = MODES[next_mode]();
                next_mode_time = Instant::now() + Duration::from_secs(MODE_CHANGE_SEC);
            }

            mode.render(0, &mut wheel_leds, &framestate)?;
            mode.render(1, &mut wheel_leds, &framestate)?;
            mode.step(&framestate)?;
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

fn render_other_stopped_mode(
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
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

const MODES: &[fn() -> Box<dyn Mode>] = &[
    // discrete-like modes
    mode_cellular::construct_cellular,
    stateless_mode!(render_graycode_rim),
    stateless_mode!(render_random_rim),
    // pulsing modes
    mode_edge_strobe::construct_edge_strobe,
    stateless_mode!(render_fade_quarters),
    stateless_mode!(render_radial_stripes),
    stateless_mode!(render_rgb_trio),
    // speckle modes
    stateless_mode!(mode_speckles::render_mod_speckle),
    stateless_mode!(mode_speckles::render_speckle_onepix),
    stateless_mode!(mode_speckles::render_speckle_random),
    stateless_mode!(mode_speckles::render_rainbow_speckle),
    // text modes
    stateless_mode!(mode_bitmap_text::render_bitmap),
    mode_bitmap_text::construct_phrase_mode,
    mode_bitmap_text::construct_speedo_mode,
    // solid image-like modes
    stateless_mode!(render_centre_red),
    stateless_mode!(render_europa),
    // rainbows and squiggles
    stateless_mode!(render_helix),
    stateless_mode!(render_pulsed_rainbow),
    stateless_mode!(render_rainbow_rim),
    stateless_mode!(render_fade_spirals),
    stateless_mode!(render_sine_full),
    stateless_mode!(render_sine),
    stateless_mode!(render_rainbows),
    stateless_mode!(render_sliders),
    mode_rainbow::construct_rainbow_on_off,
    stateless_mode!(render_fib_concentric),
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

// renders the middle pixels on each side bright red, with the
// edges (outer and hubwards) fading down to black
fn render_fib_concentric(
    side: usize,
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
    // establish a blank canvas
    for led in 0..23 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    let amber = (128, 16, 0);
    wheel_leds.set(side, 23 - 1, amber);
    wheel_leds.set(side, 23 - 2, amber);
    wheel_leds.set(side, 23 - 3, amber);
    wheel_leds.set(side, 23 - 5, amber);
    wheel_leds.set(side, 23 - 8, amber);
    wheel_leds.set(side, 23 - 13, amber);
    wheel_leds.set(side, 23 - 21, amber);

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
        wheel_leds.set(
            side,
            led as usize,
            (pix_brightness_red, pix_brightness_green, 0),
        );
    }

    Ok(())
}
