use crate::leds;
use crate::structs::{FrameState, Mode};
use crate::helpers::fraction_to_rgb;
use std::io;
use rand::Rng;

struct Trails {
    leds: [(u8, u8, u8); 23],
}

impl Mode for Trails {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, self.leds[led]);
        }
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
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

pub fn construct_white_trails() -> Box<dyn Mode> {
    Box::new(Trails { leds:[(0,0,0); 23] })
}

struct HueTrails {
    leds: [(f32, f32); 23],
}

impl Mode for HueTrails {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            let (h, v) = self.leds[led];
            leds.set(side, led, fraction_to_rgb(h,Some(v)));
        }
        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        for led in 0..23 {
            let (h, v) = self.leds[led];
            // divisor here is something that looks visually good
            self.leds[led] = (h, v/1.3);
        }

        let led = rand::thread_rng().gen_range(0, 23);
        self.leds[led] = (frame.spin_pos, 1.0); 

        Ok(())
    }
}

pub fn construct_hue_trails() -> Box<dyn Mode> {
    Box::new(HueTrails { leds:[(0.0, 0.0); 23] })
}
