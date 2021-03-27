use crate::helpers::fraction_to_rgb;
use crate::leds;
use crate::structs::{FrameState, Mode};
use rand::Rng;
use std::io;

struct RainbowOnOff {
    colours: [(bool, f32); 23],
}

impl<const LEDS: usize> Mode<LEDS> for RainbowOnOff {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let (active, hue) = self.colours[led];
            if active {
                leds.set(side, led, fraction_to_rgb(hue, None));
            } else {
                leds.set(side, led, (0, 0, 0));
            }
        }

        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let led = rand::thread_rng().gen_range(0, 23);

        let action = rand::thread_rng().gen_range(0, 5);

        if action == 0 {
            self.colours[led] = (false, 0.0);
        } else {
            let hue = if led > 0 && fst(self.colours[led - 1]) {
                snd(self.colours[led - 1])
            } else {
                rand::thread_rng().gen_range(0.0, 1.0)
            };

            self.colours[led] = (true, hue);
        }

        Ok(())
    }
}

fn fst((a, _): (bool, f32)) -> bool {
    a
}
fn snd((_, b): (bool, f32)) -> f32 {
    b
}

pub fn construct_rainbow_on_off<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(RainbowOnOff {
        colours: [(false, 0.0); 23],
    })
}
