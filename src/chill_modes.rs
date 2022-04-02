use std::io;

use crate::helpers::fraction_to_rgb;
use crate::leds::{Side, WheelLEDs};
use crate::structs::FrameState;

const MODE_CHANGE_SEC: u64 = 60;

fn chill_modes<const LEDS: usize>() -> &'static [for<'r, 's> fn(
    side: Side,
    &'r mut WheelLEDs<LEDS>,
    &'s FrameState,
) -> Result<(), std::io::Error>] {
    &[rainbow,
      complement_sides]
}

pub fn render_chill_mode<const LEDS: usize>(
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let modes = chill_modes();
    let t = (framestate.now.as_secs() / MODE_CHANGE_SEC % (modes.len() as u64)) as usize;

    let mode = modes[t];

    mode(Side::Left, wheel_leds, framestate)?;
    mode(Side::Right, wheel_leds, framestate)?;
    Ok(())
}

fn rainbow<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {

    let now_ms = framestate.now.as_millis();
    let now_steps = (now_ms as f32) / 30000.0;

    let time_phase = now_steps % 1.0;

    for led in 0..LEDS {
        let pos_phase = (led as f32) / (LEDS as f32);
        let phase = (pos_phase + time_phase) % 1.0;
        let rgb = fraction_to_rgb(phase, Some(0.5));
        wheel_leds.set(side, led, rgb);
    }

    Ok(())
}

fn complement_sides<const LEDS: usize>(
    side: Side,
    wheel_leds: &mut WheelLEDs<LEDS>,
    framestate: &FrameState,
) -> io::Result<()> {
    let side_phase = if side == Side::Left { 0.0 } else { 0.5 };

    let now_ms = framestate.now.as_millis();
    let now_steps = (now_ms as f32) / 30000.0;

    let time_phase = now_steps % 1.0;

    let phase = (time_phase + side_phase) % 1.0;

    let rgb = fraction_to_rgb(phase, Some(0.5));

    for led in 0..LEDS {
        wheel_leds.set(side, led, rgb);
    }

    Ok(())
}
