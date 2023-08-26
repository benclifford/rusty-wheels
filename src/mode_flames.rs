use crate::helpers::blank_leds;
use crate::structs::FrameState;
use rand::Rng;
use std::io;

use crate::structs::RGB24;

pub fn render_hub_white(wheel_leds: &mut [RGB24], _framestate: &FrameState) -> io::Result<()> {
    blank_leds(wheel_leds);

    let mut n = 0;

    while (n < 22) && rand::thread_rng().gen_range(0, 1000) > 250  {
        n += 1;
    }

    if n < 23 {
        wheel_leds[n] = (255, 255, 255);
    }

    Ok(())
}

