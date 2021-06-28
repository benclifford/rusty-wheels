use std::io;
use rand::Rng;

use crate::leds::{Side, WheelLEDs, SIDES};
use crate::structs::FrameState;

const MODE_CHANGE_SEC: u64 = 60;

pub fn render_stopped_mode<const LEDS: usize>(wheel_leds: &mut WheelLEDs<LEDS>, framestate: &FrameState) -> io::Result<()> {
    let t = framestate.now.as_secs() / MODE_CHANGE_SEC % 2;
    match t {
        // 0 => render_stopped_mode_red_yellow_one_random(wheel_leds, framestate),
        0 => render_stopped_mode_red_yellow_slide(wheel_leds, framestate),
        _ => render_stopped_mode_red_yellow_centre_pulse(wheel_leds, framestate),
    }
}

fn render_stopped_mode_red_yellow_one_random<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
    _framestate: &FrameState,
) -> io::Result<()> {
    let rand_led = rand::thread_rng().gen_range(0, LEDS);
    let ran_col = rand::thread_rng().gen_range(0, 2);
    for led in 0..LEDS {
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

fn render_stopped_mode_red_yellow_slide<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let this_frame_shift = ((framestate.now.as_millis() / 100) % (LEDS as u128)) as usize;

    let mut set = |l: usize, col: (u8, u8, u8)| {
        let led = (l + this_frame_shift) % LEDS;
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
    for offset in 18..LEDS {
        set(offset, (0, 0, 0));
        set(offset, (0, 0, 0));
    }

    Ok(())
}

fn render_stopped_mode_red_yellow_centre_pulse<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
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
        for led in 21..LEDS {
            wheel_leds.set(*side, led, (2, 0, 0));
        }
    }

    Ok(())
}
