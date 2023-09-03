use crate::helpers::blank_leds;
use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::Rng;
use std::io;

use crate::helpers::fraction_to_rgb;
use crate::structs::RGB24;

pub fn render_hub_white(wheel_leds: &mut [RGB24], _framestate: &FrameState) -> io::Result<()> {
    blank_leds(wheel_leds);

    let mut n = 0;

    while (n < 22) && rand::thread_rng().gen_range(0, 1000) > 250 {
        n += 1;
    }

    if n < 23 {
        wheel_leds[n] = (255, 255, 255);
    }

    Ok(())
}

struct HubRainbow {
    offset: f32,
}

pub fn create_hub_rainbow<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    let offset = rand::thread_rng().gen_range(0.0, 1.0);
    Box::new(HubRainbow { offset: offset })
}

impl<const LEDS: usize> Mode<LEDS> for HubRainbow {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        let wheel_leds = leds.side_slice(side);

        blank_leds(wheel_leds);

        let mut n = 0;

        while (n < 22) && rand::thread_rng().gen_range(0, 1000) > 250 {
            n += 1;
        }
        if n < 23 {
            let frac = ((n as f32 / 23.0) + self.offset) % 1.0;
            wheel_leds[n] = fraction_to_rgb(frac, None);
        }

        Ok(())
    }
}
