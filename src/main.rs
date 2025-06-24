use signal_hook::flag;

use std::cmp;
use std::env;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use std::time::{Duration, Instant};

use rusty_wheels::chill_modes::render_chill_mode;
use rusty_wheels::leds;
use rusty_wheels::leds::{WheelLEDs, SIDES};
use rusty_wheels::magnet::Magnet;
use rusty_wheels::moving_modes::modes;
use rusty_wheels::stopped_modes::render_caution_mode;
use rusty_wheels::structs::{FrameState, Mode};

use rusty_wheels::jumble::Jumbler;

use rusty_wheels::buttons::PushButton;

/// The duration between magnet pulses that distinguishes between
/// stopped mode and live mode.
const STOP_TIME_MS: u128 = 2000;

/// The duration between mode changes.
const MODE_CHANGE_SEC: u64 = 20;

/// The number of LEDs on each side
const N_LEDS: usize = 23;

#[derive(PartialEq)]
enum StoppedMode {
    StoppedCaution,
    StoppedWhite,
    StoppedChill,
}

impl StoppedMode {
    fn next(&self) -> StoppedMode {
        match self {
            StoppedMode::StoppedCaution => StoppedMode::StoppedWhite,
            StoppedMode::StoppedWhite => StoppedMode::StoppedChill,
            StoppedMode::StoppedChill => StoppedMode::StoppedCaution,
        }
    }
}

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

    let wheel_leds: WheelLEDs<N_LEDS> = WheelLEDs::new();

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    match run_leds(magnet, wheel_leds, push_button, shutdown_flag) {
        Ok(_) => println!("runleds finished ok"),
        Err(e) => println!("runleds returned an error: {}", e),
    }

    println!("Ending rusty-wheels");
}

fn run_leds<const LEDS: usize>(
    mut magnet: Magnet,
    mut wheel_leds: WheelLEDs<LEDS>,
    mut push_button: PushButton,
    shutdown_flag: Arc<AtomicBool>,
) -> io::Result<()> {
    let start_time = Instant::now();

    let mut spin_start_time = start_time;
    let mut last_spin_start_time = start_time;

    for side in SIDES.iter() {
        for led in 0..LEDS {
            wheel_leds.set(*side, led, (0, 0, 0));
        }
    }
    wheel_leds.show()?;

    let args: Vec<String> = env::args().collect();

    let mut loop_counter: u32 = 0;

    flag::register(signal_hook::SIGTERM, Arc::clone(&shutdown_flag))?;
    flag::register(signal_hook::SIGINT, Arc::clone(&shutdown_flag))?;

    let mut floodlight: StoppedMode = StoppedMode::StoppedCaution;

    let mut next_mode_time = Instant::now();

    let mut jumbler = Jumbler::new(modes().to_vec());

    // this is going to get replaced pretty much right away unless I implement a count-down timer mode switcher rather than
    // absolute time based phasing. But it's better than threading Option behaviour all the way through.
    let mut mode: Box<dyn Mode<LEDS>> = if args.len() <= 1 {
        jumbler.next().unwrap()()
    } else {
        let forced_mode: usize = args[1].parse().expect("parseable mode on command line");
        (modes()[forced_mode])()
    };

    let mut stats_num_frames: u32 = 0;
    let mut stats_start_time = Instant::now();

    while !(shutdown_flag.load(Ordering::Relaxed)) {
        if magnet.pulsed() {
            last_spin_start_time = spin_start_time;
            spin_start_time = Instant::now()
        };

        if push_button.pulsed() {
            println!("push button pulse");
            floodlight = floodlight.next();
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

        if (mode_duration.as_millis() > STOP_TIME_MS || mode_duration.as_millis() == 0) && args.len() <= 1 {
            match floodlight {
                StoppedMode::StoppedCaution => render_caution_mode(&mut wheel_leds, &framestate),
                StoppedMode::StoppedWhite => render_floodlight_mode(&mut wheel_leds, &framestate),
                StoppedMode::StoppedChill => render_chill_mode(&mut wheel_leds, &framestate),
            }?;
        } else {
            if next_mode_time <= Instant::now() && args.len() <= 1 {
                mode = (jumbler.next().unwrap())();
                next_mode_time = Instant::now() + Duration::from_secs(MODE_CHANGE_SEC);

                let stats_duration = stats_start_time.elapsed();
                let stats_fps = (stats_num_frames as f32) / (stats_duration.as_secs() as f32);
                println!(
                    "Frame rate statistics: {} frames over {:?} = {} frames/s",
                    stats_num_frames, stats_duration, stats_fps
                );
                stats_num_frames = 0;
                stats_start_time = Instant::now();
            }

            mode.pre_step(&framestate)?;
            mode.render(leds::Side::Left, &mut wheel_leds, &framestate)?;
            mode.render(leds::Side::Right, &mut wheel_leds, &framestate)?;
            mode.step(&framestate)?;
        }

        wheel_leds.show()?;

        loop_counter += 1;
        stats_num_frames += 1;
    }
    let duration_secs = start_time.elapsed().as_secs();
    println!("Duration {} seconds", duration_secs);

    // run a shutdown effect

    for side in SIDES.iter() {
        for led in 0..LEDS {
            wheel_leds.set(*side, led, (1, 1, 1));
        }
    }
    wheel_leds.show()?;

    thread::sleep(Duration::from_millis(250));

    for side in SIDES.iter() {
        for led in 0..LEDS {
            wheel_leds.set(*side, led, (0, 0, 0));
        }
    }
    wheel_leds.show()?;

    println!("ending");
    Ok(())
}

fn render_floodlight_mode<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
    _framestate: &FrameState,
) -> io::Result<()> {
    for side in SIDES.iter() {
        for led in 0..LEDS {
            wheel_leds.set(*side, led, (32, 32, 32));
        }
        // override the middle ones with full brightness
        for led in 9..14 {
            wheel_leds.set(*side, led, (255, 255, 255));
        }
    }

    Ok(())
}
