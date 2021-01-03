use crate::helpers::spinpos_to_rgb;
use crate::leds::{Side, WheelLEDs};
use crate::structs::FrameState;
use rand::Rng;
use std::io;

pub fn render_mod_speckle(
    side: Side,
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

pub fn render_speckle_onepix(
    side: Side,
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

pub fn render_speckle_random(
    side: Side,
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

pub fn render_rainbow_speckle(
    side: Side,
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
