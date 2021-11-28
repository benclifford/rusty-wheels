use std::io;

use crate::leds::{Side, WheelLEDs};
use crate::structs::FrameState;

const MODE_CHANGE_SEC: u64 = 60;

fn stopped_modes<const LEDS: usize>() -> &'static [for<'r, 's> fn(
    side: Side,
    &'r mut WheelLEDs<LEDS>,
    &'s FrameState,
) -> Result<(), std::io::Error>] {
    &[
        amber_quarters_fader,
        amber_quarters,
        amber_swap,
        red_yellow_slide,
        red_yellow_centre_pulse,
        full_quick_pulse,
    ]
}

pub fn render_stopped_mode<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let modes = stopped_modes();
    let t = (framestate.now.as_secs() / MODE_CHANGE_SEC % (modes.len() as u64)) as usize;

    let mode = modes[t];

    mode(Side::Left, wheel_leds, framestate)?;
    mode(Side::Right, wheel_leds, framestate)?;
    Ok(())
}

fn red_yellow_slide<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let this_frame_shift = ((framestate.now.as_millis() / 32) % (LEDS as u128)) as usize;

    let mut set = |l: usize, col: (u8, u8, u8)| {
        let led = l + this_frame_shift;
        if led < LEDS {
            wheel_leds.set(side, led, col);
        }
    };

    set(0, (0, 0, 0));

    for offset in 1..7 {
        set(offset, (255, 0, 0));
    }
    for offset in 7..13 {
        set(offset, (0, 0, 0));
    }
    for offset in 13..19 {
        set(offset, (255, 64, 0));
    }
    for offset in 19..LEDS {
        set(offset, (0, 0, 0));
    }

    Ok(())
}

fn full_quick_pulse<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_millis = framestate.now.as_millis();
    let now_secs = framestate.now.as_secs();
    let flicker = (now_millis / 25) % 4 == 0 && (now_millis / 250) % 2 == 0;
    let topside = now_secs % 2 == 0;
    if topside ^ (side == Side::Left) {
        for led in 0..LEDS {
            if flicker {
                wheel_leds.set(side, led, (255, 64, 0));
            } else {
                wheel_leds.set(side, led, (0, 0, 0));
            }
        }
    } else {
        for led in 9..14 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    }

    Ok(())
}

fn red_yellow_centre_pulse<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_millis = framestate.now.as_millis();
    let now_secs = framestate.now.as_secs();
    let flicker = (now_millis / 25) % 4 == 0;
    let topside = now_secs % 2 == 0;
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

    if topside ^ (side == Side::Left) {
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
    for led in 21..LEDS {
        wheel_leds.set(side, led, (2, 0, 0));
    }

    Ok(())
}

fn amber_quarters<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_ms = framestate.now.as_millis();

    let a = now_ms % 2000;

    let segs = a / 1000; // range: 0 .. 1

    let flip = (segs == 0) ^ (side == Side::Left);

    if flip {
        for led in 0..11 {
            wheel_leds.set(side, led, (255, 64, 0));
        }
        for led in 11..LEDS {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    } else {
        for led in 0..11 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
        for led in 11..LEDS {
            wheel_leds.set(side, led, (255, 64, 0));
        }
    }

    Ok(())
}

fn amber_quarters_fader<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_ms = framestate.now.as_millis();

    let a = now_ms % 2000; // 0 .. 2000

    let quarter = a / 1000; // 0 .. 1
    let cycle = a % 1000;

    let updown = cycle / 500;
    let fadecycle = cycle % 500;

    let s2 = (fadecycle * 255 / 500) as u8; // rescale to 0..255
    let s3 = (fadecycle * 64 / 500) as u8; // rescale to 0..64

    let on_col = if updown == 0 {
        (s2, s3, 0)
    } else {
        (255 - s2, 64 - s3, 0)
    };

    let flip = (quarter == 0) ^ (side == Side::Left);

    if flip {
        for led in 0..11 {
            wheel_leds.set(side, led, on_col);
        }
        for led in 11..LEDS {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    } else {
        for led in 0..11 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
        for led in 11..LEDS {
            wheel_leds.set(side, led, on_col);
        }
    }

    Ok(())
}

fn amber_swap<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let now_secs = framestate.now.as_secs();
    let topside = now_secs % 2 == 0;
    for led in 0..8 {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    if topside ^ (side == Side::Left) {
        for led in 9..14 {
            wheel_leds.set(side, led, (255, 64, 0));
        }
    } else {
        for led in 9..14 {
            wheel_leds.set(side, led, (0, 0, 0));
        }
    }
    for led in 14..LEDS {
        wheel_leds.set(side, led, (0, 0, 0));
    }

    Ok(())
}
