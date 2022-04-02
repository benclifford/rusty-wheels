use std::io;

use crate::helpers::fraction_to_rgb;
use crate::leds::{Side, WheelLEDs};
use crate::structs::FrameState;

const MODE_CHANGE_SEC: u64 = 10;

fn chill_modes<const LEDS: usize>() -> &'static [for<'r, 's> fn(
    side: Side,
    &'r mut WheelLEDs<LEDS>,
    &'s FrameState,
) -> Result<(), std::io::Error>] {
    &[rainbow]
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
    for led in 0..LEDS {
        let phase = (led as f32) / (LEDS as f32);
        let rgb = fraction_to_rgb(phase, Some(0.5));
        wheel_leds.set(side, led, rgb);
    }

    Ok(())
}
