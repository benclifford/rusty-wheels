mod buttons;
mod helpers;
mod leds;
mod magnet;
mod mode_bitmap_text;
mod mode_cellular;
mod mode_dither;
mod mode_edge_strobe;
mod mode_misc;
mod mode_oval;
mod mode_rainbow;
mod mode_randomwalk;
mod mode_rgb_dither;
mod mode_speckles;
mod mode_trails;
mod structs;

use signal_hook::flag;

use std::cmp;
use std::env;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use std::time::{Duration, Instant};

use rand::Rng;

use leds::{Side, WheelLEDs, SIDES};
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

    for side in SIDES.iter() {
        for led in 0..23 {
            wheel_leds.set(*side, led, (0, 0, 0));
        }
    }
    wheel_leds.show()?;

    let args: Vec<String> = env::args().collect();

    let mut loop_counter: u32 = 0;

    flag::register(signal_hook::SIGTERM, Arc::clone(&shutdown_flag))?;
    flag::register(signal_hook::SIGINT, Arc::clone(&shutdown_flag))?;

    // floodlight mode: when false, stopped mode should be a floodlight
    // when true, stopped mode should be animated traffic caution modes
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
                render_floodlight_mode(&mut wheel_leds, &framestate)?;
            }
        } else {
            if next_mode_time <= Instant::now() && args.len() <= 1 {
                let next_mode = rand::thread_rng().gen_range(0, MODES.len());
                mode = MODES[next_mode]();
                next_mode_time = Instant::now() + Duration::from_secs(MODE_CHANGE_SEC);
            }

            mode.pre_step(&framestate)?;
            mode.render(leds::Side::Left, &mut wheel_leds, &framestate)?;
            mode.render(leds::Side::Right, &mut wheel_leds, &framestate)?;
            mode.step(&framestate)?;
        }

        wheel_leds.show()?;

        loop_counter += 1;
    }
    let duration_secs = start_time.elapsed().as_secs();
    println!("Duration {} seconds", duration_secs);

    // run a shutdown effect

    for side in SIDES.iter() {
        for led in 0..23 {
            wheel_leds.set(*side, led, (1, 1, 1));
        }
    }
    wheel_leds.show()?;

    thread::sleep(Duration::from_millis(250));

    for side in SIDES.iter() {
        for led in 0..23 {
            wheel_leds.set(*side, led, (0, 0, 0));
        }
    }
    wheel_leds.show()?;

    println!("ending");
    Ok(())
}

fn render_floodlight_mode(wheel_leds: &mut WheelLEDs, _framestate: &FrameState) -> io::Result<()> {
    for side in SIDES.iter() {
        for led in 0..23 {
            wheel_leds.set(*side, led, (32, 32, 32));
        }
        // override the middle ones with full brightness
        for led in 9..14 {
            wheel_leds.set(*side, led, (255, 255, 255));
        }
    }

    Ok(())
}

fn render_stopped_mode(wheel_leds: &mut WheelLEDs, framestate: &FrameState) -> io::Result<()> {
    let t = framestate.now.as_secs() / 20 % 3;
    match t {
        0 => render_stopped_mode_red_yellow_one_random(wheel_leds, framestate),
        1 => render_stopped_mode_red_yellow_slide(wheel_leds, framestate),
        _ => render_stopped_mode_red_yellow_centre_pulse(wheel_leds, framestate),
    }
}

fn render_stopped_mode_red_yellow_one_random(
    wheel_leds: &mut WheelLEDs,
    _framestate: &FrameState,
) -> io::Result<()> {
    let rand_led = rand::thread_rng().gen_range(0, 23);
    let ran_col = rand::thread_rng().gen_range(0, 2);
    for led in 0..23 {
        wheel_leds.set(Side::Left, led, (0, 0, 0));
        wheel_leds.set(Side::Right, led, (0, 0, 0));
    }

    let rcol = if ran_col == 0 {
        (255, 0, 0)
    } else {
        (255, 128, 0)
    };

    wheel_leds.set(Side::Left, rand_led, rcol);
    wheel_leds.set(Side::Right, rand_led, rcol);

    Ok(())
}

fn render_stopped_mode_red_yellow_slide(
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    let this_frame_shift = ((framestate.now.as_millis() / 100) % 23) as usize;

    let mut set = |l: usize, col: (u8, u8, u8)| {
        let led = (l + this_frame_shift) % 23;
        wheel_leds.set(Side::Left, led, col);
        wheel_leds.set(Side::Right, led, col);
    };

    for offset in 0..6 {
        set(offset, (255, 0, 0));
    }
    for offset in 6..12 {
        set(offset, (0, 0, 0));
        set(offset, (0, 0, 0));
    }
    for offset in 12..18 {
        set(offset, (255, 128, 0));
        set(offset, (255, 128, 0));
    }
    for offset in 18..23 {
        set(offset, (0, 0, 0));
        set(offset, (0, 0, 0));
    }

    Ok(())
}

fn render_stopped_mode_red_yellow_centre_pulse(
    wheel_leds: &mut WheelLEDs,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_millis = framestate.now.as_millis();
    let now_secs = framestate.now.as_secs();
    let flicker = (now_millis / 25) % 4 == 0;
    let topside = now_secs % 2 == 0;
    for side in &SIDES {
        for led in 0..2 {
            wheel_leds.set(*side, led, (2, 0, 0));
        }
        for led in 2..4 {
            wheel_leds.set(*side, led, (8, 0, 0));
        }
        for led in 4..6 {
            wheel_leds.set(*side, led, (64, 0, 0));
        }

        for led in 6..8 {
            wheel_leds.set(*side, led, (255, 0, 0));
        }

        for led in 8..9 {
            wheel_leds.set(*side, led, (0, 0, 0));
        }

        if topside ^ (*side == Side::Left) {
            for led in 9..14 {
                if flicker {
                    wheel_leds.set(*side, led, (255, 255, 0));
                } else {
                    wheel_leds.set(*side, led, (0, 0, 0));
                }
            }
        } else {
            for led in 9..14 {
                wheel_leds.set(*side, led, (0, 0, 0));
            }
        }
        for led in 14..15 {
            wheel_leds.set(*side, led, (0, 0, 0));
        }
        for led in 15..17 {
            wheel_leds.set(*side, led, (255, 0, 0));
        }
        for led in 17..19 {
            wheel_leds.set(*side, led, (64, 0, 0));
        }
        for led in 19..21 {
            wheel_leds.set(*side, led, (8, 0, 0));
        }
        for led in 21..23 {
            wheel_leds.set(*side, led, (2, 0, 0));
        }
    }

    Ok(())
}

const MODES: &[fn() -> Box<dyn Mode>] = &[
    stateless_mode!(mode_oval::render_oval),
    mode_bitmap_text::construct_phrase_mode_hello,
    mode_rgb_dither::create_dither,
    mode_dither::create_dither,
    mode_trails::construct_hue_trails_sparse,
    mode_trails::construct_hue_trails,
    mode_trails::construct_white_trails,
    mode_randomwalk::create_float_spray,
    mode_randomwalk::create_random_walk_dot,
    // discrete-like modes
    mode_cellular::construct_cellular,
    stateless_mode!(mode_misc::render_graycode_rim),
    stateless_mode!(mode_misc::render_random_rim),
    stateless_mode!(mode_misc::render_random_rim_red_yellow),
    // pulsing modes
    mode_edge_strobe::construct_edge_strobe,
    stateless_mode!(mode_misc::render_fade_quarters),
    stateless_mode!(mode_misc::render_radial_stripes),
    stateless_mode!(mode_misc::render_rgb_trio),
    // speckle modes
    stateless_mode!(mode_speckles::render_mod_speckle),
    stateless_mode!(mode_speckles::render_speckle_onepix),
    stateless_mode!(mode_speckles::render_speckle_random),
    stateless_mode!(mode_speckles::render_rainbow_speckle),
    // text modes
    mode_bitmap_text::construct_phrase_fuck_boris,
    mode_bitmap_text::construct_phrase_mode,
    mode_bitmap_text::construct_speedo_mode,
    // solid image-like modes
    stateless_mode!(mode_misc::render_centre_red),
    stateless_mode!(mode_misc::render_europa),
    // rainbows and squiggles
    stateless_mode!(mode_misc::render_helix),
    stateless_mode!(mode_misc::render_pulsed_rainbow),
    stateless_mode!(mode_misc::render_rainbow_rim),
    stateless_mode!(mode_misc::render_fade_spirals),
    stateless_mode!(mode_misc::render_sine_full),
    stateless_mode!(mode_misc::render_sine),
    stateless_mode!(mode_misc::render_rainbows),
    stateless_mode!(mode_misc::render_sliders),
    mode_rainbow::construct_rainbow_on_off,
    stateless_mode!(mode_misc::render_fib_concentric),
];
