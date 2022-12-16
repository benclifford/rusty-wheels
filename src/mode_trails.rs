use crate::helpers::fraction_to_rgb;
use crate::leds;
use crate::structs::{FrameState, Mode, RGB24};
use rand::Rng;
use std::io;

struct Trails {
    leds: [RGB24; 23],
}

impl<const LEDS: usize> Mode<LEDS> for Trails {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, self.leds[led]);
        }
        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        let led = rand::thread_rng().gen_range(0, 23);
        let swiz = rand::thread_rng().gen_range(0, 2);
        if swiz == 0 {
            self.leds[led] = (0, 0, 0);
        } else {
            self.leds[led] = (255, 255, 255);
        }
        Ok(())
    }
}

pub fn construct_white_trails<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(Trails {
        leds: [(0, 0, 0); 23],
    })
}

struct HueTrails {
    trigger_denominator: usize,
    leds: [(f32, f32); 23],
}

impl<const LEDS: usize> Mode<LEDS> for HueTrails {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let (h, v) = self.leds[led];
            leds.set(side, led, fraction_to_rgb(h, Some(v)));
        }
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        for led in 0..23 {
            let (h, v) = self.leds[led];
            // divisor here is something that looks visually good
            self.leds[led] = (h, v / 1.3);
        }

        if rand::thread_rng().gen_range(0, self.trigger_denominator) == 0 {
            let led = rand::thread_rng().gen_range(0, 23);
            self.leds[led] = (frame.spin_pos, 1.0);
        } // else don't turn on anything

        Ok(())
    }
}

pub fn construct_hue_trails<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(HueTrails {
        trigger_denominator: 1,
        leds: [(0.0, 0.0); 23],
    })
}

pub fn construct_hue_trails_sparse<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    Box::new(HueTrails {
        trigger_denominator: 5,
        leds: [(0.0, 0.0); 23],
    })
}
